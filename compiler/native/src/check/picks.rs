// Template call picks (docs/ARCH.md section 4.1, `CallTrait`). A
// template body is checked once against its bounds (the template
// check, `generics::template_check`). Every call there that
// resolves through a trait method records a pick: the trait, and
// where the Self type sits relative to the receiver. Every other
// check of the same body (concrete instances and closure instances)
// replays the pick: the trait is fixed, and selection on the Self
// type there picks the impl. Method lookup never runs again for those calls, so
// an instance cannot see the inherent methods, builtin std methods,
// or other traits of a concrete type that the template could not
// see (rustc also resolves methods once per generic body).
//
// The template check also needs rustc's rule that the methods of a
// type parameter's where clauses are inherent candidates
// (`where_clause_probe`): they win over a blanket impl of another
// trait at the same probe, and they match by receiver type, so a
// by-value probe at `&T` takes a `&self` bound method of `T`.
//
// Operators (`a + b`, `-a`, `a += b`) and `for` loops re-select in
// instances. An operator names exactly one lang trait, so selection
// in an instance finds the impl coherence allows for the operand
// types (or the builtin scalar path); a bound that fixes `Output` is
// checked again at the call site. The one open input is a literal
// operand: `t == 5` under `T: PartialEq<u8>` must stay `u8` in an
// instance whose Self also has `PartialEq<u16>`, so a binary operator
// a param-env predicate proved records that predicate
// (`TraitPick::env_pred`) and instances unify the operands with it
// first (`replay_operands`). A `for`
// loop is the same shape through `IntoIterator`: the blanket
// `impl<I: Iterator> IntoIterator for I` overlaps any direct
// impl, so the template proof and the instance selection name
// the same impl. The tests `operator_pick_*` and
// `for_loop_pick_*` in tests/pass pin both shapes.
use super::method::{self, Probe, RcvProbe};
use super::*;
use crate::def::DefId;

/// One call a template check resolved through a trait method.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TraitPick {
    pub trait_def: DefId,
    /// Method calls: autoderefs from the receiver to the Self type.
    /// Zero for path calls (`T::f(..)`).
    pub derefs: u8,
    /// Method calls: the method's receiver form on Self.
    pub form: ReceiverForm,
    /// True when Self is reached by the array-to-slice autoderef
    /// step after `derefs` peels (the receiver is a `SliceOf` of
    /// the array place, `Autoref::SliceRef`).
    pub slice: bool,
    /// The template env predicate that proved the call (a
    /// `ParamEvidence` pick), by index into the template's env. Other
    /// checks replay that trait ref with its inputs (`U: From<u8>`
    /// gives `u16: From<u8>`, never an open `u16: From<?X>`), as rustc
    /// resolves the call once per generic body. Bound order is part of
    /// the template's own `Sig` fingerprint, so a reused pick's index
    /// still names the same predicate.
    pub env_pred: Option<u32>,
}

impl Ctx<'_> {
    /// True when this body is a template check, whose picks are
    /// recorded rather than replayed.
    pub(super) fn is_template_check(&self) -> bool {
        self.p.gx.template_checks.contains_key(&self.fid)
    }

    /// The pick a template check recorded for one call, when this
    /// body is any other check of that template's body.
    pub(super) fn replay_pick(&self, e: ExprId) -> Option<TraitPick> {
        if self.records_picks() {
            return None;
        }
        // A revealed check replays its own opaque view's picks first
        // (check/opaque.rs).
        self.view_picks.get(&e).or_else(|| self.p.gx.trait_picks.get(&e)).copied()
    }

    /// Records one confirmed trait call in a template check.
    pub(super) fn record_pick(&mut self, e: ExprId, trait_def: DefId, derefs: u8, form: ReceiverForm) {
        self.record_pick_env(e, trait_def, derefs, form, None);
    }

    /// `record_pick` for a call a param-env predicate proved: `env` is
    /// its index in this body's env (`TraitPick::env_pred`).
    /// `usize::MAX` is selection's structural answer, not a predicate.
    pub(super) fn record_pick_env(&mut self, e: ExprId, trait_def: DefId, derefs: u8, form: ReceiverForm, env: Option<usize>) {
        self.record_pick_full(e, trait_def, derefs, form, env, false);
    }

    /// Records a pick a method probe confirmed: the probe carries
    /// the array-to-slice step in its autoref.
    pub(super) fn record_pick_probe(&mut self, e: ExprId, trait_def: DefId, derefs: u8, probe: RcvProbe, env: Option<usize>) {
        let slice = matches!(probe.autoref, Autoref::SliceRef | Autoref::SliceRefMut);
        self.record_pick_full(e, trait_def, derefs, probe.filter, env, slice);
    }

    fn record_pick_full(&mut self, e: ExprId, trait_def: DefId, derefs: u8, form: ReceiverForm, env: Option<usize>, slice: bool) {
        self.last_pick = Some((e, trait_def));
        if self.records_picks() {
            // An index past the fn's own env names an opaque type's bound
            // (`Ctx::solve_env`): the revealed check selects on the hidden
            // type instead.
            let env_pred = env.filter(|&i| i < self.env().len()).and_then(|i| u32::try_from(i).ok());
            self.out.trait_picks.push((e, TraitPick { trait_def, derefs, form, slice, env_pred }));
        }
    }

    /// The trait ref a replayed param-env pick of trait `trait_def`
    /// names in this instance: the template's env predicate with the
    /// instance's args substituted, its projections normalized.
    pub(super) fn replayed_trait_ref(&mut self, e: ExprId, trait_def: DefId) -> Option<TraitRef> {
        let pick = self.replay_pick(e).filter(|pk| pk.trait_def == trait_def)?;
        let i = pick.env_pred? as usize;
        let p = self.p;
        let info = p.gx.instances.get(&self.fid)?;
        let template = info.template_local?;
        let Some(&Predicate::Trait(b)) = p.gx.fn_scopes.get(template.0 as usize)?.env.get(i) else { return None };
        if b.count != 0 || b.value.def != trait_def {
            return None;
        }
        let inst: Vec<GenericArg> = p.tys.args(info.args).to_vec();
        let tr = crate::traits::index::subst_trait_ref_pub(&mut self.tys, b.value, &inst);
        let raw: Vec<GenericArg> = self.tys.args(tr.args).to_vec();
        let normed: Vec<GenericArg> = raw
            .into_iter()
            .map(|g| match g {
                GenericArg::Ty(t) => GenericArg::Ty(self.normalize_here(t)),
                other => other,
            })
            .collect();
        let args = self.tys.intern_args(&normed);
        Some(TraitRef { args, ..tr })
    }

    /// An operator's operand types against the template's param-env
    /// pick, in an instance: the template proved `T: PartialEq<u8>`
    /// for `t == 5`, so the literal is `u8` here too, whatever other
    /// impls the instance's Self type has.
    /// All or nothing: operands of another shape (references the
    /// template peeled) bind nothing.
    pub(super) fn replay_operands(&mut self, e: ExprId, trait_def: DefId, operands: &[Ty]) {
        let Some(r) = self.replayed_trait_ref(e, trait_def) else { return };
        let want: Vec<Ty> = self.tys.args(r.args).iter().filter_map(|g| g.as_ty()).collect();
        let snap = self.snapshot_vars();
        if !operands.iter().zip(&want).all(|(&have, &w)| self.unify(have, w)) {
            self.rollback_vars(snap);
        }
    }

    /// `replay_operands` with the trait the pick names, before an
    /// operator's builtin scalar path can see an open operand.
    pub(super) fn replay_pick_operands(&mut self, e: ExprId, operands: &[Ty]) {
        if let Some(pick) = self.replay_pick(e) {
            self.replay_operands(e, pick.trait_def, operands);
        }
    }

    /// The traits of the where clauses on one type parameter (or
    /// projection) that declare a method `name` with a template, from
    /// this body's env. The env is elaborated, so supertraits count.
    /// Only a param's bounds bring a trait in scope (rustc's inherent
    /// candidates); a projection or opaque type keeps the bound traits
    /// that are in scope anyway.
    pub(super) fn bound_traits_with(&mut self, self_ty: Ty, name: Sym) -> Vec<DefId> {
        let p = self.p;
        let Some(traits) = p.gx.traits_by_method.get(&name) else { return Vec::new() };
        let self_ty = self.resolve(self_ty);
        let param = match self.tys.kind(self_ty) {
            TyKind::Param(_) => true,
            TyKind::Projection(..) | TyKind::Opaque(..) => false,
            _ => return Vec::new(),
        };
        let env = self.solve_env();
        let is_bound = |t: DefId| {
            env.iter().any(|pred| {
                matches!(pred, Predicate::Trait(b) if b.count == 0 && b.value.def == t
                    && self.tys.args(b.value.args).first().and_then(|g| g.as_ty()) == Some(self_ty))
            })
        };
        let mut out: Vec<DefId> = Vec::new();
        for &t in traits {
            let templated = p.gx.index.trait_record(t).and_then(|r| r.method_by_name(name)).is_some_and(|m| p.gx.trait_methods.contains_key(&m));
            if templated && !out.contains(&t) && is_bound(t) {
                out.push(t);
            }
        }
        if !param {
            out.retain(|&t| method::trait_in_scope_pub(self, t));
        }
        out
    }
}

/// The receiver form a trait's method `name` takes on Self, from its
/// template. None for associated fns, typed receivers, and lang
/// trait methods (no template).
fn method_form(cx: &Ctx, trait_def: DefId, name: Sym) -> Option<ReceiverForm> {
    let p = cx.p;
    let m = p.gx.index.trait_record(trait_def)?.method_by_name(name)?;
    match p.gx.trait_methods.get(&m)?.self_kind? {
        ast::SelfKind::Value => Some(ReceiverForm::Value),
        ast::SelfKind::Ref => Some(ReceiverForm::Ref),
        ast::SelfKind::RefMut => Some(ReceiverForm::RefMut),
        ast::SelfKind::Typed => None,
    }
}

/// The where-clause probe of one receiver-type probe variant
/// (rustc's inherent candidates from the param env): the methods of
/// the bounds on the variant's Self type whose receiver form is the
/// variant's form. The pointee variant of a by-value probe at `&T`
/// covers a `&self` bound method of `T`. Two bound traits with a
/// matching method are ambiguous (rustc E0034).
#[allow(clippy::too_many_arguments)]
pub(super) fn where_clause_probe(
    cx: &mut Ctx,
    e: ExprId,
    probe: RcvProbe,
    derefs: u8,
    name: Sym,
    name_span: Span,
    args: &[ExprId],
    explicit: &[Option<GenericArg>],
    expected: Option<Ty>,
) -> Probe {
    let self_ty = cx.resolve(probe.self_ty);
    cx.note_opaques(self_ty);
    let traits: Vec<DefId> =
        cx.bound_traits_with(probe.self_ty, name).into_iter().filter(|&t| method_form(cx, t, name) == Some(probe.filter)).collect();
    match traits.len() {
        0 => Probe::Miss,
        1 => method::trait_probe_for(cx, e, Some(traits[0]), &[probe], derefs, name, name_span, args, explicit, expected),
        _ => {
            cx.err(
                "L0720",
                name_span,
                format!("multiple trait methods named `{}` apply here", cx.name(name)),
                "Call the method with a fully qualified path, such as `<Type as Trait>::method(...)`.",
            );
            for &a in args {
                cx.expr(a, None);
            }
            Probe::Failed
        }
    }
}

/// Replays one recorded pick at a method call: Self sits
/// `pick.derefs` derefs into the receiver, and only the picked trait
/// is probed, at the recorded form. Selection on this body's Self
/// type picks the impl. A lang trait impl with no method fn (a builtin
/// type, a derive) has a builtin body: None tells the caller to use
/// the builtin dispatch only, never an inherent method. Any other miss
/// means the bound did not hold where the instance was requested
/// (that call site already reported it), or that the template check
/// and this check disagree: either way it is the no-impl error.
#[allow(clippy::too_many_arguments)]
pub(super) fn replay_method(
    cx: &mut Ctx,
    e: ExprId,
    recv_ty: Ty,
    pick: TraitPick,
    name: Sym,
    name_span: Span,
    args: &[ExprId],
    explicit: &[Option<GenericArg>],
    expected: Option<Ty>,
) -> Option<Ty> {
    // The same autoderef steps as the template's search: references,
    // then `Box` and `Rc` (`method::generic_method_search`).
    let mut step = cx.resolve(recv_ty);
    let mut ok = true;
    for _ in 0..pick.derefs {
        match cx.tys.kind(step) {
            TyKind::Ref(_, _, inner) => step = cx.resolve(inner),
            _ => match cx.tys.smart_elem(step) {
                Some(inner) => step = cx.resolve(inner),
                None => ok = false,
            },
        }
    }
    // A call through an `Into<U>` bound returns U: in an instance the
    // bound names it even where nothing else expects a type
    // (`t.into().0`), and the `From` impl to bridge to depends on it.
    let expected = match expected {
        Some(t) if !matches!(cx.tys.kind(cx.resolve(t)), TyKind::Infer(_)) => Some(t),
        _ if pick.trait_def == cx.p.gx.lang.into => cx.instance_bound_arg(pick.trait_def, step).or(expected),
        _ => expected,
    };
    if ok && pick.slice {
        // The template's step unsized an array to a slice.
        match cx.tys.kind(step) {
            TyKind::Array(el, _) => step = cx.tys.intern(TyKind::Slice(el)),
            _ if step == STRING => step = STR,
            _ => match cx.tys.vec_elem(step) {
                Some(el) => step = cx.tys.intern(TyKind::Slice(el)),
                None => ok = false,
            },
        }
    }
    if ok {
        let probe = if pick.slice {
            RcvProbe {
                self_ty: step,
                recv_ty: cx.tys.mk_ref(pick.form == ReceiverForm::RefMut, step),
                autoref: if pick.form == ReceiverForm::RefMut { Autoref::SliceRefMut } else { Autoref::SliceRef },
                extra_derefs: 0,
                filter: pick.form,
            }
        } else {
            RcvProbe::at(cx, step, pick.form)
        };
        // A `?Sized` param instantiated at a dyn type whose principal (or
        // a supertrait of it) is the picked trait: the object candidate,
        // through the vtable (ARCH 2.8), never an `impl dyn` method.
        if matches!(cx.tys.kind(step), TyKind::Dyn(..)) {
            match method::dyn_probe(cx, e, probe, pick.derefs, name, name_span, args, &[], Some(pick.trait_def)) {
                Probe::Hit(ret) => return Some(ret),
                Probe::Failed => return Some(ERROR),
                Probe::Miss => {}
            }
        }
        match method::trait_probe_for(cx, e, Some(pick.trait_def), &[probe], pick.derefs, name, name_span, args, explicit, expected) {
            Probe::Hit(ret) => return Some(ret),
            Probe::Failed => return Some(ERROR),
            Probe::Miss => {}
        }
        // A `Clone` pick with no impl fn (a reference, a builtin type):
        // the builtin clone of exactly the step the template picked, so
        // `t.clone()` on `t: &T` with an unbounded `T` copies the reference.
        let lang = cx.p.gx.lang;
        if pick.trait_def == lang.clone
            && cx.name(name) == "clone"
            && pick.form == ReceiverForm::Ref
            && !pick.slice
            && args.is_empty()
            && super::has_clone(cx.p, &cx.tys, step)
        {
            cx.set_res(e, Res::BuiltinMethod { b: Builtin::Clone, derefs: pick.derefs, autoref: Autoref::Ref });
            return Some(step);
        }
        // `impl AsRef<U> for &T where T: AsRef<U>` (and `&mut T`; AsMut
        // through `&mut T`) has no body in lust: it forwards to T's impl,
        // so a pick that landed on a reference to a type with a user
        // impl calls that impl one deref further (W-SHIM: `P:
        // AsRef<Path>` at `&PathBuf` or `&str`).
        let (mut step, mut derefs) = (step, pick.derefs);
        while pick.trait_def == lang.as_ref || pick.trait_def == lang.as_mut {
            let TyKind::Ref(_, m, inner) = cx.tys.kind(step) else { break };
            if pick.trait_def == lang.as_mut && !m.is_mut() {
                break;
            }
            (step, derefs) = (cx.resolve(inner), derefs + 1);
            let probe = RcvProbe::at(cx, step, pick.form);
            match method::trait_probe_for(cx, e, Some(pick.trait_def), &[probe], derefs, name, name_span, args, explicit, expected) {
                Probe::Hit(ret) => return Some(ret),
                Probe::Failed => return Some(ERROR),
                Probe::Miss => {}
            }
        }
        // core's `impl<I: Iterator + ?Sized> Iterator for &mut I` (and the
        // DoubleEndedIterator and ExactSizeIterator ones) have no body in
        // lust either: their `next`, `size_hint`, `nth`, `next_back`,
        // `nth_back`, `len` forward to `I`, so a pick at `&mut U` with a
        // user impl for `U` calls that impl one deref further (`i.next()`
        // in an instance with I = &mut UserIter, also &mut &mut UserIter).
        let forwards = !pick.slice
            && match cx.p.interner.get(name) {
                "next" | "size_hint" | "nth" => pick.trait_def == lang.iterator,
                "next_back" | "nth_back" => pick.trait_def == lang.double_ended_iterator,
                "len" => pick.trait_def == lang.exact_size_iterator,
                _ => false,
            };
        while forwards && let TyKind::Ref(_, Mut::Mut, inner) = cx.tys.kind(step) {
            (step, derefs) = (cx.resolve(inner), derefs + 1);
            let probe = RcvProbe::at(cx, step, pick.form);
            match method::trait_probe_for(cx, e, Some(pick.trait_def), &[probe], derefs, name, name_span, args, explicit, expected) {
                Probe::Hit(ret) => return Some(ret),
                Probe::Failed => return Some(ERROR),
                Probe::Miss => {}
            }
        }
    }
    let p = cx.p;
    let templated =
        p.gx.index.trait_record(pick.trait_def).and_then(|r| r.method_by_name(name)).is_some_and(|m| p.gx.trait_methods.contains_key(&m));
    if ok && !templated {
        return None;
    }
    for &a in args {
        cx.expr(a, None);
    }
    if cx.kind(step) != TyKind::Error {
        let tn = cx.tn(step);
        let tr_name = cx.trait_name(pick.trait_def);
        cx.err("L0612", name_span, format!("no impl of `{tr_name}` for `{tn}`"), "Add the impl, or change the bound.");
    }
    Some(ERROR)
}

#[cfg(test)]
mod tests {
    use crate::check::collect::{self, Program};
    use crate::def::{CrateIds, LOCAL_CRATE};
    use crate::diag::FileId;
    use crate::intern::{Interner, Known};
    use crate::{lexer, parser};

    /// Collects one sample file in build (`walk` true) or check mode,
    /// then runs `run` on the program.
    fn with_mode(src: &str, walk: bool, run: impl FnOnce(&Interner, &Program)) {
        let interner = std::sync::Arc::new(Interner::default());
        let known = std::sync::Arc::new(Known::new(&interner));
        let lexed = lexer::lex(src, FileId(0), 0, &interner).unwrap();
        let ast = parser::parse(&lexed, &interner, &known).unwrap();
        let ids = CrateIds { local: LOCAL_CRATE, stable: crate::def::synthetic_crate_hash("test.rs", false) };
        let program = collect::collect_mode(std::sync::Arc::new(ast), interner.clone(), known.clone(), ids, walk).unwrap();
        run(&interner, &program);
    }

    /// `with_mode` for a build.
    fn with_program(src: &str, run: impl FnOnce(&Interner, &Program)) {
        with_mode(src, true, run);
    }

    /// The template check records the call through the bound with the
    /// bound's trait and Self one deref into `&T`, by `&self`.
    #[test]
    fn template_check_records_the_bound_trait() {
        let src = "struct S;\nimpl S { fn get(&self) -> u8 { 1 } }\ntrait G { fn get(&self) -> u8; }\n\
                   impl G for S { fn get(&self) -> u8 { 2 } }\nfn via<T: G>(t: &T) -> u8 { t.get() }\n\
                   fn main() { println!(\"{}\", via(&S)); }\n";
        with_program(src, |interner, p| {
            let g = p.gx.trait_names[&interner.intern("G")];
            let picks: Vec<_> = p.gx.trait_picks.values().collect();
            assert_eq!(picks.len(), 1);
            assert_eq!(picks[0].trait_def, g);
            assert_eq!((picks[0].derefs, picks[0].form), (1, super::ReceiverForm::Ref));
        });
    }

    /// The lang signature table is built from the lang trait
    /// records: `PartialOrd::partial_cmp` exists with an
    /// `Option<Ordering>` return, the comparison defaults exist,
    /// and methods a bound cannot call (`Hash::hash`,
    /// `Debug::fmt`) have no entry.
    #[test]
    fn lang_sigs_come_from_the_records() {
        use crate::ty::TyIntern as _;
        let src = "fn main() { println!(\"x\"); }\n";
        with_program(src, |interner, p| {
            let lang = &p.gx.lang;
            let sig_of = |trait_def, name: &str| {
                let rec = p.gx.index.trait_record(trait_def).expect("lang record");
                let m = rec.method_by_name(interner.intern(name)).expect("record method");
                p.gx.lang_method_sigs.get(&m)
            };
            let pc = sig_of(lang.partial_ord, "partial_cmp").expect("partial_cmp sig");
            assert_eq!(pc.self_kind, Some(crate::ast::SelfKind::Ref));
            let elem = p.tys.option_elem(pc.ret).expect("Option return");
            assert_eq!(p.tys.adt_parts(elem).map(|(d, _)| d), Some(crate::check::generics::ordering_def()));
            assert!(sig_of(lang.partial_ord, "lt").is_some());
            assert!(sig_of(lang.ord, "cmp").is_some());
            assert!(sig_of(lang.add, "add").is_some());
            assert!(sig_of(lang.add_assign, "add_assign").is_some());
            assert!(sig_of(lang.hash, "hash").is_none());
            // `fmt` has its sig since user fmt impls (W-STD-B): `(&self,
            // &mut Formatter) -> fmt::Result`.
            let fmt = sig_of(lang.debug, "fmt").expect("fmt sig");
            assert_eq!(fmt.params.len(), 2);
            assert!(sig_of(lang.drop, "drop").is_none());
        });
    }

    /// Derived glue calls a written impl only where the glue models
    /// rustc's calls exactly (`glue_methods`): non-generic impls on
    /// non-generic types; not a `Copy` type's `Clone`, not a
    /// `PartialEq` that writes `ne`, not a `PartialOrd` next to an
    /// `Ord` impl or with its own `lt`.
    #[test]
    fn glue_methods_take_only_modelled_impls() {
        let src = "use std::cmp::Ordering;\n\
                   struct A(u8);\nimpl Clone for A { fn clone(&self) -> A { A(self.0) } }\n\
                   #[derive(Copy)] struct B(u8);\nimpl Clone for B { fn clone(&self) -> B { *self } }\n\
                   struct C(u8);\nimpl PartialEq for C { fn eq(&self, o: &C) -> bool { self.0 == o.0 } fn ne(&self, o: &C) -> bool { self.0 != o.0 } }\n\
                   struct D(u8);\nimpl PartialEq for D { fn eq(&self, o: &D) -> bool { self.0 == o.0 } }\n\
                   impl PartialOrd for D { fn partial_cmp(&self, o: &D) -> Option<Ordering> { self.0.partial_cmp(&o.0) } }\n\
                   #[derive(PartialEq, Eq)] struct F(u8);\n\
                   impl PartialOrd for F { fn partial_cmp(&self, o: &F) -> Option<Ordering> { Some(self.cmp(o)) } }\n\
                   impl Ord for F { fn cmp(&self, o: &F) -> Ordering { self.0.cmp(&o.0) } }\n\
                   struct G<T>(T);\nimpl<T> Default for G<T> { fn default() -> G<T> { loop {} } }\n\
                   struct H(u8);\nimpl Default for H { fn default() -> H { H(1) } }\n\
                   fn main() {}\n";
        with_program(src, |interner, p| {
            let lang = &p.gx.lang;
            let has = |n: &str, tr| p.gx.glue_methods.contains_key(&(p.type_names[&interner.intern(n)], tr));
            assert!(has("A", lang.clone));
            assert!(!has("B", lang.clone));
            assert!(!has("C", lang.partial_eq));
            assert!(has("D", lang.partial_eq));
            assert!(has("D", lang.partial_ord));
            assert!(!has("F", lang.partial_ord));
            assert!(!has("G", lang.default));
            assert!(has("H", lang.default));
        });
    }

    /// `lust check` makes closure body fns (they check closure bodies)
    /// but no generic fn instances; a build makes both (ARCH 4.1).
    #[test]
    fn check_mode_makes_no_generic_fn_instances() {
        use crate::check::generics::FnClass;
        let src = "fn id<T>(t: T) -> T { t }\nfn main() { let f = |x: u8| x + 1; println!(\"{} {}\", id(5u8), f(1)); }\n";
        let count = |p: &Program, k: FnClass| p.gx.fn_class.iter().filter(|&&c| c == k).count();
        with_mode(src, true, |_, p| assert_eq!((count(p, FnClass::Instance), count(p, FnClass::Closure)), (1, 1)));
        with_mode(src, false, |_, p| assert_eq!((count(p, FnClass::Instance), count(p, FnClass::Closure)), (0, 1)));
    }
}
