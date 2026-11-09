// Return-position `impl Trait` (docs/ARCH.md 2.11).
//
// Collect gives each `fn f(..) -> impl Bounds` an opaque type
// `Opaque(def, args)` over every param in scope (edition 2024 captures
// them all; lifetimes are erased here and the borrow checker treats the
// call conservatively). Its bounds are stored over the fn's params with
// the opaque itself as one more param (`OpaqueInfo`), so a use
// substitutes `args ++ [self]`.
//
// The defining body checks against a fresh variable in place of its own
// opaque: the returns decide the hidden type, which must be fully known,
// must not name the opaque itself, and must satisfy every bound. The
// hidden type is stored over the defining fn's params
// (`GenTables::opaque_hidden`, from `reveal_plain` and the template
// check), and a plain fn's `FnInfo::ret` is revealed to it for codegen
// while callers keep the opaque (`Program::ret_view`).
//
// Callers (W-CHECK-G3) see only the opaque type and its bounds, as in
// rustc. A body that uses an opaque value is checked twice:
//
// - The opaque view (`OpaqueMode::View`): an opaque type is rigid; its
//   elaborated bounds join the env selection and normalization use
//   (`note_opaques`), so it has exactly the traits and associated types
//   its bounds name, plus the auto traits its hidden type has (rustc's
//   auto-trait leakage). Trait calls on it record picks, as a template's
//   calls do. Errors come from this check.
// - The revealed check (`OpaqueMode::Revealed`): every callee's opaque
//   return is its hidden type (`reveal`), and the calls replay the view's
//   picks, so the hidden type's inherent methods and other traits stay
//   out of reach. Lowering, borrowck, and codegen get this check.
//
// Instances check revealed and replay their template's picks. A template
// that uses an opaque value checks in the view; lowering marks its
// opaque-holding locals by their view types, so its borrow check tracks
// the value through the capture var as a revealed body does
// (fx-bench4-gaps).
use super::*;
use crate::intern::FxMap;

impl Ctx<'_> {
    /// The return type a body checks against: the fn's return type, with
    /// its own `impl Trait` as a fresh variable the returns decide.
    pub(super) fn defining_ret(&mut self) -> Ty {
        let p = self.p;
        let ret = p.ret_view(self.fid);
        let TyKind::Opaque(od, args) = self.tys.kind(ret) else { return ret };
        let owner = p.gx.instances.get(&self.fid).map_or_else(|| p.def_id(self.fid), |i| i.template_def);
        if p.gx.opaques.get(&od).is_none_or(|o| o.owner != owner) {
            return ret;
        }
        let v = self.new_var(false);
        self.own_opaques.push((od, args, v));
        v
    }

    /// `t` itself, or the variable standing for its hidden type when `t`
    /// is an opaque type this body defines.
    pub(super) fn map_own_opaque(&self, t: Ty) -> Ty {
        match self.tys.kind(t) {
            TyKind::Opaque(od, args) => self.own_opaques.iter().find(|(d, a, _)| *d == od && *a == args).map_or(t, |&(_, _, v)| v),
            _ => t,
        }
    }

    /// Queues the bounds of the body's own opaque types as obligations
    /// on their hidden types. Binders are region-erased here, as the
    /// body's other types are.
    pub(super) fn push_opaque_bounds(&mut self, span: Span) {
        let p = self.p;
        for (od, args, v) in self.own_opaques.clone() {
            let Some(info) = p.gx.opaques.get(&od) else { continue };
            let mut full: Vec<GenericArg> = self.tys.args(args).to_vec();
            full.push(GenericArg::Ty(v));
            for &pred in &info.bounds {
                match crate::traits::index::subst_predicate(&mut self.tys, pred, &full) {
                    Predicate::Trait(b) => self.obl.push(obligations::Obligation::Trait { trait_ref: b.value, span, depth: 0 }),
                    Predicate::AssocEq(b) => self.obl.push(obligations::Obligation::AssocEq { eq: b.value, span }),
                    Predicate::TypeOutlives(ty, Region::Static) => self.obl.push(obligations::Obligation::TypeStatic { ty, span }),
                    _ => {}
                }
            }
        }
    }

    /// Records the hidden type of each own opaque after body end, and
    /// rejects one the body did not decide or that names itself.
    pub(super) fn finish_opaques(&mut self, span: Span) {
        let p = self.p;
        for (od, _, v) in std::mem::take(&mut self.own_opaques) {
            let h = self.deep(v);
            if self.tys.flags(h).contains_infer() {
                self.err(
                    "L0610",
                    span,
                    "type annotations needed: the returned value's type behind `impl Trait` is not known",
                    "Annotate the returned value's type.",
                );
                continue;
            }
            if mentions_opaque(&self.tys, h, Some(od)) {
                self.err(
                    "L0110",
                    span,
                    "cannot resolve the `impl Trait` type: the returned value's type names it again (a recursive fn)",
                    "Return a concrete type, or box the recursion behind a trait object.",
                );
                continue;
            }
            if self.closure_not_general(od, h) {
                self.err(
                    "L0735",
                    span,
                    "the returned closure is not general enough for its `impl Fn*` type: an unannotated parameter takes one inferred lifetime, but the type is for any lifetime",
                    "Write the closure as the returned value itself, or annotate its parameter (`|a: &u8|`).",
                );
                continue;
            }
            if self.ptr_not_general(od, h) || self.item_not_general(od, h) {
                self.err(
                    "L0735",
                    span,
                    "the returned fn pointer is not general enough for its `impl Fn*` type: it returns a lifetime of its own where the type returns its argument's",
                    "Return the closure itself, or a fn pointer whose output lifetime is elided (`fn(&T) -> &U`).",
                );
                continue;
            }
            self.out.opaque_hidden.push((od, h));
        }
    }

    /// True when hidden type `h` of opaque `od` is a closure checked with
    /// no expected signature (`free_sig_closures`) that cannot meet an
    /// `Fn*` bound of the opaque (rustc "implementation of `Fn` is not
    /// general enough"): an unannotated param holds a reference where the
    /// bound takes a higher-ranked one (`let f = |a| a; f`), or its output
    /// holds a region where the bound's output is the input's (its own
    /// output region is one inferred region, never the input's). The
    /// bound's `'static` output slots become the closure's
    /// `output_region` otherwise.
    fn closure_not_general(&mut self, od: DefId, h: Ty) -> bool {
        let TyKind::Closure(def, _) = self.kind(h) else { return false };
        let unannotated = self.free_sig_closures.iter().find(|c| c.0 == def).map(|c| c.1.clone()).unwrap_or_default();
        let p = self.p;
        let Some(info) = p.gx.opaques.get(&od) else { return false };
        let Some((inputs, output, free_out)) =
            self.closures.iter().chain(&self.out.closures).find(|c| c.def == def).map(|c| (c.inputs.clone(), c.output, c.free_sig))
        else {
            return false;
        };
        if !free_out {
            return false;
        }
        let regions = |this: &mut Self, t: Ty| {
            let t = this.deep(t);
            let mut ts = Vec::new();
            crate::borrowck::regions_of(&this.tys, t, &mut ts);
            ts
        };
        for b in &info.fn_outs {
            if b.inputs.len() != inputs.len() {
                continue;
            }
            for (i, &bt) in b.inputs.iter().enumerate() {
                let mut rs = Vec::new();
                crate::borrowck::regions_of(&p.tys, bt, &mut rs);
                if unannotated.get(i).copied().unwrap_or(false) && rs.iter().any(|r| matches!(r, Region::Bound(_))) && !regions(self, inputs[i]).is_empty() {
                    return true;
                }
            }
            let Some(bo) = b.output else { continue };
            let mut outs = Vec::new();
            crate::borrowck::regions_of(&p.tys, bo, &mut outs);
            let out_d = self.deep(output);
            if outs.iter().any(|r| matches!(r, Region::Bound(_))) && crate::check::closure::holds_region_slots(&p.gx.adt_lifetimes, &self.tys, out_d) {
                return true;
            }
            if outs.contains(&Region::Static)
                && let Some(c) = self.out.closures.iter_mut().find(|c| c.def == def)
                && c.output_region.is_none()
            {
                c.output_region = Some(bo);
            }
        }
        false
    }

    /// True when hidden type `h` of opaque `od` is a fn pointer that
    /// cannot meet an `Fn*` bound of the opaque whose output is the
    /// input's: an output slot there is `'static` in the pointer (`fn(&u8)
    /// -> &'static u8`, a join of closures with free outputs) or its free
    /// lifetime (rustc "implementation of `Fn` is not general enough").
    fn ptr_not_general(&mut self, od: DefId, h: Ty) -> bool {
        let TyKind::FnPtr(s) = self.kind(h) else { return false };
        let s = self.tys.sig(s).value;
        let p = self.p;
        let Some(info) = p.gx.opaques.get(&od) else { return false };
        let ins: Vec<Ty> = self.tys.tys_of(s.inputs).to_vec();
        let mut n_in = 0;
        for &t in &ins {
            let mut rs = Vec::new();
            crate::borrowck::regions_of(&self.tys, t, &mut rs);
            n_in += rs.len();
        }
        let mut prs = Vec::new();
        crate::borrowck::regions_of(&self.tys, s.output, &mut prs);
        for b in &info.fn_outs {
            let Some(bo) = b.output.filter(|_| b.inputs.len() == ins.len()) else { continue };
            let mut brs = Vec::new();
            crate::borrowck::regions_of(&p.tys, bo, &mut brs);
            if brs.len() != prs.len() {
                continue;
            }
            let lost = brs.iter().zip(&prs).enumerate().any(|(k, (br, pr))| {
                matches!(br, Region::Bound(_)) && (*pr == Region::Static || s.free_slots >> (n_in + k) & 1 == 1)
            });
            if lost {
                return true;
            }
        }
        false
    }

    /// True when hidden type `h` of opaque `od` is a fn item whose output
    /// slot is `'static`, or a lifetime that is early-bound or no input's,
    /// where an `Fn*` bound of the opaque returns its input's lifetime
    /// (rustc "implementation of `Fn` is not general enough").
    fn item_not_general(&mut self, od: DefId, h: Ty) -> bool {
        let TyKind::FnDef(def, _) = self.kind(h) else { return false };
        let p = self.p;
        let (Some(info), Some(sig)) = (p.gx.opaques.get(&od), p.local_def(def.hash).and_then(|l| p.regions.body_sig(p, l))) else { return false };
        let mut ins = Vec::new();
        for &t in &sig.params {
            crate::borrowck::regions_of(&p.tys, t, &mut ins);
        }
        let mut outs = Vec::new();
        crate::borrowck::regions_of(&p.tys, sig.ret, &mut outs);
        let fixed = |r: Region| match r {
            Region::Static => true,
            Region::Param(k) => sig.early.get(k as usize).copied().unwrap_or(false) || !ins.contains(&r),
            _ => false,
        };
        info.fn_outs.iter().any(|b| {
            let Some(bo) = b.output.filter(|_| b.inputs.len() == sig.params.len()) else { return false };
            let mut brs = Vec::new();
            crate::borrowck::regions_of(&p.tys, bo, &mut brs);
            brs.len() == outs.len() && brs.iter().zip(&outs).any(|(br, &r)| matches!(br, Region::Bound(_)) && fixed(r))
        })
    }

    /// Rejects a value of an opaque type in a body that gets code
    /// (L0602, see the file header): a revealed check that still has one
    /// (a hidden type lust does not know). Views may name them.
    pub(super) fn reject_opaque_uses(&mut self) {
        let p = self.p;
        if p.gx.opaques.is_empty() || self.is_template_check() || self.opaque_mode == OpaqueMode::View {
            return;
        }
        let n = self.out.expr_ty.len();
        for i in 0..n {
            let t = self.out.expr_ty[i];
            if mentions_opaque(&self.tys, t, None) {
                let e = ExprId(self.out.expr_base + i as u32);
                let span = self.span(e);
                self.errs.push(not_yet("L0602", span, "using the value of a fn that returns `impl Trait`"));
                return;
            }
        }
    }

    /// Adds the elaborated bounds of every opaque type in `t` this body
    /// has not met yet to its solve env (`Ctx::solve_env`), so selection
    /// and normalization answer for it as for a type parameter.
    pub(super) fn note_opaques(&mut self, t: Ty) {
        let p = self.p;
        if p.gx.opaques.is_empty() || !mentions_opaque(&self.tys, t, None) {
            return;
        }
        let mut found: Vec<Ty> = Vec::new();
        collect_opaques(&self.tys, t, &mut found);
        for o in found {
            if self.opaque_seen.contains(&o) {
                continue;
            }
            self.opaque_seen.push(o);
            let TyKind::Opaque(od, args) = self.tys.kind(o) else { continue };
            let Some(info) = p.gx.opaques.get(&od) else { continue };
            let mut full: Vec<GenericArg> = self.tys.args(args).to_vec();
            full.push(GenericArg::Ty(o));
            let written: Vec<Predicate> =
                info.bounds.iter().map(|&pred| crate::traits::index::subst_predicate(&mut self.tys, pred, &full)).collect();
            let more = crate::traits::solve::elaborate_in(&mut self.tys, &p.gx.index, &written);
            if self.opaque_env.is_empty() {
                self.opaque_env = self.env().to_vec();
            }
            for pred in more {
                if !self.opaque_env.contains(&pred) {
                    self.opaque_env.push(pred);
                }
            }
        }
    }

    /// A callee's return type as this check sees it: unchanged in the
    /// opaque view, with every opaque type replaced by its hidden type in
    /// a revealed check (`reveal`). Records the `Sig` read of each
    /// revealed opaque's defining fn (its fingerprint covers the hidden
    /// type), so a daemon rechecks this body when the hidden type moves.
    pub(super) fn callee_ret(&mut self, t: Ty) -> Ty {
        if self.opaque_mode != OpaqueMode::Revealed || !mentions_opaque(&self.tys, t, None) {
            return t;
        }
        let p = self.p;
        let mut found: Vec<Ty> = Vec::new();
        collect_opaques_deep(p.gx.opaque_hidden.as_map(), &mut self.tys, t, &mut found);
        for o in found {
            if let TyKind::Opaque(od, _) = self.tys.kind(o)
                && let Some(info) = p.gx.opaques.get(&od)
            {
                let _ = p.sig_of(&mut self.out.reads, info.owner);
                if let TyKind::Opaque(_, args) = self.tys.kind(o)
                    && let Some(&h) = p.gx.opaque_hidden.get(&od)
                {
                    // The closures of this opaque's own hidden type (not of
                    // the opaques nested in it: `found` has those with their
                    // own args), as the full reveal names them.
                    let a: Vec<GenericArg> = self.tys.args(args).to_vec();
                    let h = subst(&mut self.tys, h, &a);
                    let mut cts = Vec::new();
                    collect_closures(&self.tys, h, &mut cts);
                    for ct in cts {
                        let ct = self.reveal_here(ct);
                        if let TyKind::Closure(d, _) = self.tys.kind(ct)
                            && !self.opaque_closure_args.contains(&(d, args, ct))
                        {
                            self.opaque_closure_args.push((d, args, ct));
                        }
                    }
                }
            }
        }
        // A generic definer's hidden closures are its template's; the
        // instance this call runs has its own, which body end maps in
        // once the args are known (`build_closure_map`).
        if !self.opaque_calls.contains(&t) {
            self.opaque_calls.push(t);
        }
        self.reveal_here(t)
    }

    /// Body end: the closure map (`instance_closure_map`) of each generic
    /// opaque the revealed calls name, at its resolved args, for an
    /// instance mono made (none exists before mono). `zonk` applies it.
    /// Two instances whose hidden closures have the same type (`T` only
    /// in the body, not the captures) cannot be told apart: L0602.
    pub(super) fn build_closure_map(&mut self) {
        let p = self.p;
        let mut found: Vec<Ty> = Vec::new();
        // A literal's var (`mk(5)`) takes its default now, as the zonk
        // right after would: mono made the instance at that type.
        let mut bare: Vec<Ty> = Vec::new();
        for t in std::mem::take(&mut self.opaque_calls) {
            self.default_literal_vars(t);
            let t = self.deep(t);
            collect_opaques_deep(p.gx.opaque_hidden.as_map(), &mut self.tys, t, &mut found);
            let r = reveal(p.gx.opaque_hidden.as_map(), &mut self.tys, t, 0);
            bare_iter_args(&self.tys, r, &mut bare);
        }
        // The inner iterators the call types became once their args
        // resolved (`reveal_here`); `zonk` replaces them first.
        self.iter_arg_map = self.canon_iter_args(bare);
        let mut map: Vec<(Ty, DefId)> = Vec::new();
        // Per generic opaque met: its args and its hidden closure types.
        let mut met: Vec<(DefId, ArgsId, Vec<Ty>)> = Vec::new();
        // Innermost first: an opaque's args can hold the value of
        // another (`pass(def(4u8))` with `fn pass(it: impl Iterator)`),
        // whose closures are mapped before this instance is looked up.
        for o in found.into_iter().rev() {
            let TyKind::Opaque(od, args) = self.tys.kind(o) else { continue };
            // A method probe that lost records a call at vars nothing
            // decides; the call that stands has its args resolved.
            if self.tys.flags(o).contains_infer() {
                continue;
            }
            let Some(info) = p.gx.opaques.get(&od) else { continue };
            let Some(template) = p.local_def(info.owner.hash) else { continue };
            if p.gx.fn_templates.get(template.0 as usize).is_none_or(|t| t.is_none()) {
                continue;
            }
            let h = self.reveal_here(o);
            let mut cts = Vec::new();
            collect_closures(&self.tys, h, &mut cts);
            if cts.is_empty() || met.iter().any(|(d, a, _)| *d == od && *a == args) {
                continue;
            }
            if met.iter().any(|(d, _, c)| *d == od && c.iter().any(|t| cts.contains(t))) {
                let span = p.fn_def(self.fid).name_span;
                self.errs.push(not_yet(
                    "L0602",
                    span,
                    "two calls of one generic `impl Trait` fn at different type args whose hidden closures have the same type",
                ));
                return;
            }
            met.push((od, args, cts));
            let args: Vec<GenericArg> = self.tys.args(args).to_vec();
            let args: Vec<GenericArg> = args.into_iter().map(|g| map_arg_closures(&mut self.tys, g, &map)).collect();
            let Some(inst) = generics::instance_for(p, &self.tys, template, &args).filter(|&i| i != template) else {
                self.out.closure_map_pending = true;
                continue;
            };
            // An instance another body made this round is not checked
            // yet, so its closures have no kind or signature (`std::iter::
            // from_fn(..)` could not select `next`): check again after it.
            let m = instance_closure_map(p, &self.tys, h, template, p.def_id(inst));
            if m.iter().any(|(_, d)| p.gx.closures.get(d).is_none()) {
                self.out.closure_map_pending = true;
                continue;
            }
            map.extend(m);
        }
        self.closure_map = map;
    }

    /// True when this check records its trait picks for a later check of
    /// the same body: a template check, or an opaque view.
    pub(super) fn records_picks(&self) -> bool {
        self.is_template_check() || (self.opaque_mode == OpaqueMode::View && !self.p.gx.opaques.is_empty())
    }
}

/// True when an opaque type's bounds name `trait_def` for it, directly
/// or through supertraits (`impl Error` is `Display` and `Debug`). The
/// structural queries (`has_display`, `has_debug`, `is_copy`) ask this
/// where they ask the env for a type parameter.
pub(crate) fn opaque_bound_holds(p: &Program, tys: &impl TyIntern, t: Ty, trait_def: DefId) -> bool {
    let TyKind::Opaque(od, _) = tys.kind(t) else { return false };
    let Some(info) = p.gx.opaques.get(&od) else { return false };
    fn reaches(p: &Program, from: DefId, want: DefId, seen: &mut Vec<DefId>) -> bool {
        if from == want {
            return true;
        }
        if seen.contains(&from) {
            return false;
        }
        seen.push(from);
        let Some(rec) = p.gx.index.trait_record(from) else { return false };
        rec.generics.predicates.iter().any(|pred| match pred {
            Predicate::Trait(b) => {
                matches!(p.tys.args(b.value.args).first().and_then(|g| g.as_ty()).map(|s| p.tys.kind(s)), Some(TyKind::Param(ParamId(0))))
                    && reaches(p, b.value.def, want, seen)
            }
            _ => false,
        })
    }
    // Every trait predicate of an opaque's bounds has the opaque as Self.
    info.bounds.iter().any(|pred| matches!(pred, Predicate::Trait(b) if reaches(p, b.value.def, trait_def, &mut Vec::new())))
}

/// What an opaque view hands its revealed check: its trait picks and the
/// Fn trait of each call of an opaque callable.
#[derive(Default)]
pub(super) struct ViewOut {
    pub picks: FxMap<ExprId, picks::TraitPick>,
    pub calls: FxMap<ExprId, ViewCall>,
}

/// How the opaque view called an `impl Fn*` value (`ViewOut::calls`):
/// the Fn trait, the `&` layers peeled off the callee before the opaque
/// (`r()` on `r: &impl FnOnce()` is 1), and whether the opaque is `Copy`.
#[derive(Clone, Copy)]
pub(super) struct ViewCall {
    pub kind: closure::ClosureKind,
    pub refs: u8,
    pub copy: bool,
}

/// The `ViewCall` of each call in an opaque view's `closure_calls`.
fn view_calls(p: &Program, view: &Checked) -> FxMap<ExprId, ViewCall> {
    let call = |&(e, kind): &(ExprId, closure::ClosureKind)| {
        let (mut refs, mut copy) = (0u8, false);
        if let Expr::Call { callee, .. } = p.ast.expr(e) {
            let mut t = view.ty(*callee);
            while let TyKind::Ref(_, _, inner) = view.tys.kind(t) {
                refs += 1;
                t = inner;
            }
            match view.tys.kind(t) {
                TyKind::Opaque(..) => copy = opaque_bound_holds(p, &view.tys, t, p.gx.lang.copy),
                _ => refs = 0,
            }
        }
        (e, ViewCall { kind, refs, copy })
    };
    view.closure_calls.iter().map(call).collect()
}

/// How a value holds `impl Trait` values, from its type in the opaque
/// view (`view_hold`). Lowering marks the locals that hold one
/// (`Body::opaque_locals`): such a local captures every region of its
/// opaque values, and a `Drop` one also keeps them live to its drop.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Hold {
    #[default]
    None,
    /// Only opaques whose bounds say `Copy`: rustc knows they have no
    /// drop glue, so they add nothing to the value's drop.
    Copy,
    /// An opaque rustc assumes has a destructor that uses every region
    /// it captures (its drop check cannot see the hidden type).
    Drop,
}

/// How a value of the view type `t` holds opaque values: an opaque type
/// itself, or one it owns by value (a reference or a `dyn` owns none; a
/// `dyn` made from one carries its captures in its own region).
pub(crate) fn view_hold(p: &Program, tys: &impl TyIntern, t: Ty) -> Hold {
    let in_args = |a: ArgsId| tys.args(a).iter().filter_map(|g| g.as_ty()).map(|e| view_hold(p, tys, e)).max().unwrap_or_default();
    match tys.kind(t) {
        TyKind::Opaque(..) if opaque_bound_holds(p, tys, t, p.gx.lang.copy) => Hold::Copy,
        TyKind::Opaque(..) => Hold::Drop,
        // A rigid projection of an opaque (`<impl IntoIterator as
        // IntoIterator>::IntoIter`) may hold what the opaque captures.
        TyKind::Projection(..) if mentions_opaque(tys, t, None) => Hold::Drop,
        TyKind::Slice(e) | TyKind::Array(e, _) => view_hold(p, tys, e),
        TyKind::Tuple(ts) => tys.tys_of(ts).iter().map(|&e| view_hold(p, tys, e)).max().unwrap_or_default(),
        TyKind::Adt(_, a) | TyKind::Closure(_, a) => in_args(a),
        _ => Hold::None,
    }
}

/// Fills a revealed check's `Checked::opaque_holds` and `local_holds`
/// from its opaque view's types: each expression, pattern, and binding
/// that holds an opaque value in the view gives its revealed type (the
/// type lowering gives the locals that hold its value). Also the values
/// the view moves where the revealed check would copy (`expr_moves`,
/// `local_moves`), and L0602 for a closure that the hidden type's
/// `Copy` makes borrow what rustc moves into it.
fn holds_from_view(p: &Program, fid: LocalDefId, view: &Checked, ck: &mut Checked, errs: &mut Vec<Diag>) {
    use crate::check::collect::is_copy;
    let moved = |v: Ty, r: Ty| view_hold(p, &view.tys, v) != Hold::None && !is_copy(p, &view.tys, v) && is_copy(p, &ck.tys, r);
    let expr_moves: Vec<bool> = view.expr_ty.iter().zip(&ck.expr_ty).map(|(&v, &r)| moved(v, r)).collect();
    ck.expr_moves = if expr_moves.contains(&true) { expr_moves } else { Vec::new() };
    let mut holds: Vec<(Ty, Hold)> = Vec::new();
    let mut add = |t: Ty, h: Hold| {
        if h == Hold::None {
            return;
        }
        match holds.iter_mut().find(|(x, _)| *x == t) {
            Some(e) => e.1 = e.1.max(h),
            None => holds.push((t, h)),
        }
    };
    for (&v, &r) in view.expr_ty.iter().zip(&ck.expr_ty).chain(view.pat_ty.iter().zip(&ck.pat_ty)) {
        add(r, view_hold(p, &view.tys, v));
    }
    let local_holds: Vec<Hold> = view.locals.iter().map(|l| view_hold(p, &view.tys, l.ty)).collect();
    for (h, l) in local_holds.iter().zip(&ck.locals) {
        add(l.ty, *h);
    }
    // Both checks bind the same patterns in the same order; a check
    // that did not leaves the bindings to the type match.
    let same = view.locals.len() == ck.locals.len() && view.locals.iter().zip(&ck.locals).all(|(a, b)| a.name == b.name && a.span == b.span);
    ck.local_holds = if same { local_holds } else { Vec::new() };
    ck.opaque_holds = holds;
    // Without lined-up bindings no binding is known to move: a binding
    // that might is not supported yet.
    let local_moves: Vec<bool> = view.locals.iter().zip(&ck.locals).map(|(v, r)| moved(v.ty, r.ty)).collect();
    if !same && local_moves.contains(&true) {
        let span = p.fn_def(fid).name_span;
        errs.push(not_yet("L0602", span, "an `impl Trait` binding whose hidden type is `Copy` when its bounds are not, here"));
    }
    ck.local_moves = if same && local_moves.contains(&true) { local_moves } else { Vec::new() };
    // A closure the view makes `FnOnce` (it calls an `impl FnOnce` or
    // moves an opaque value out) keeps that kind, though the revealed
    // check copies the value: rustc's closure is consumed by its call.
    for c in ck.closures.iter_mut() {
        if let Some(vc) = view.closures.iter().find(|v| v.def == c.def) {
            c.kind = c.kind.max(vc.kind);
        }
    }
    // A closure the view makes take a value it moves (by value) but the
    // revealed check lets borrow it (its hidden type is `Copy`): rustc's
    // closure owns the value, lust's would not.
    for c in &ck.closures {
        let Some(vc) = view.closures.iter().find(|v| v.def == c.def) else { continue };
        for cap in &c.captures {
            let by_value = vc.captures.iter().any(|v| v.root == cap.root && v.mode == closure::CaptureMode::Value);
            if cap.mode != closure::CaptureMode::Value && by_value {
                let span = p.ast.expr_span(c.expr);
                errs.push(not_yet("L0602", span, "a closure that takes an `impl Trait` value whose hidden type is `Copy` when its bounds are not"));
                return;
            }
        }
    }
}

/// How one check sees opaque types (see the file header).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpaqueMode {
    View,
    Revealed,
}

/// `check_fn` for a body that is not a closure body: the opaque view,
/// then, for a body that gets code and uses an opaque value, the revealed
/// check that replays the view's picks. An instance checks revealed.
pub(super) fn check_fn_opaque<'p>(p: &'p Program, fid: LocalDefId) -> (Checked<'p>, Vec<Diag>) {
    use generics::FnClass;
    if p.gx.opaques.is_empty() {
        return check_fn_mode(p, fid, OpaqueMode::View, ViewOut::default());
    }
    match p.gx.fn_class.get(fid.0 as usize) {
        Some(FnClass::Instance) => check_fn_mode(p, fid, OpaqueMode::Revealed, ViewOut::default()),
        Some(FnClass::Template | FnClass::TemplateClosure | FnClass::Stub) => check_fn_mode(p, fid, OpaqueMode::View, ViewOut::default()),
        _ => {
            let (view, errs) = check_fn_mode(p, fid, OpaqueMode::View, ViewOut::default());
            let uses = view.expr_ty.iter().chain(view.pat_ty.iter()).any(|&t| mentions_opaque(&view.tys, t, None));
            if !errs.is_empty() || !uses {
                return (view, errs);
            }
            let out = ViewOut { picks: view.trait_picks.iter().copied().collect(), calls: view_calls(p, &view) };
            let (mut ck, mut errs) = check_fn_mode(p, fid, OpaqueMode::Revealed, out);
            // The revealed check replays the view's picks, so it does not ask
            // again what the view asked to make them (which traits are in
            // scope for a method call): the view's reads join the stored set.
            for (k, r) in view.reads.iter(&crate::driver::graph::ReadKeys::default()) {
                ck.reads.record(*k, r.fingerprint, r.failed);
            }
            holds_from_view(p, fid, &view, &mut ck, &mut errs);
            (ck, errs)
        }
    }
}

/// `check_fn` in the opaque view only: the hidden types of the body's
/// own opaques, before any caller is revealed (`reveal_plain`).
pub(crate) fn check_fn_view<'p>(p: &'p Program, fid: LocalDefId) -> (Checked<'p>, Vec<Diag>) {
    check_fn_mode(p, fid, OpaqueMode::View, ViewOut::default())
}

/// `t` with every opaque type replaced by its hidden type at its args,
/// again inside the result (a hidden type may name another opaque). An
/// opaque with no stored hidden type stays (the revealed check then
/// rejects the value, L0602).
pub(crate) fn reveal(hidden: &FxMap<DefId, Ty>, tys: &mut impl TyIntern, t: Ty, depth: u32) -> Ty {
    if depth > 32 || !mentions_opaque(tys, t, None) {
        return t;
    }
    match tys.kind(t) {
        TyKind::Opaque(od, a) => match hidden.get(&od) {
            Some(&h) => {
                let a = reveal_args(hidden, tys, a, depth);
                let args: Vec<GenericArg> = tys.args(a).to_vec();
                let h = subst(tys, h, &args);
                reveal(hidden, tys, h, depth + 1)
            }
            None => t,
        },
        TyKind::Ref(r, m, e) => {
            let e = reveal(hidden, tys, e, depth + 1);
            tys.intern(TyKind::Ref(r, m, e))
        }
        TyKind::RawPtr(m, e) => {
            let e = reveal(hidden, tys, e, depth + 1);
            tys.intern(TyKind::RawPtr(m, e))
        }
        TyKind::Slice(e) => {
            let e = reveal(hidden, tys, e, depth + 1);
            tys.intern(TyKind::Slice(e))
        }
        TyKind::Array(e, n) => {
            let e = reveal(hidden, tys, e, depth + 1);
            tys.intern(TyKind::Array(e, n))
        }
        TyKind::Tuple(ts) => {
            let old: Vec<Ty> = tys.tys_of(ts).to_vec();
            let new: Vec<Ty> = old.into_iter().map(|e| reveal(hidden, tys, e, depth + 1)).collect();
            let ts = tys.intern_tys(&new);
            tys.intern(TyKind::Tuple(ts))
        }
        TyKind::Adt(d, a) => {
            let a = reveal_args(hidden, tys, a, depth);
            tys.intern(TyKind::Adt(d, a))
        }
        TyKind::FnDef(d, a) => {
            let a = reveal_args(hidden, tys, a, depth);
            tys.intern(TyKind::FnDef(d, a))
        }
        TyKind::Closure(d, a) => {
            let a = reveal_args(hidden, tys, a, depth);
            tys.intern(TyKind::Closure(d, a))
        }
        TyKind::Projection(d, a) => {
            let a = reveal_args(hidden, tys, a, depth);
            tys.intern(TyKind::Projection(d, a))
        }
        _ => t,
    }
}

/// The closure types of a generic definer's hidden type `t`, each
/// mapped to the def of the same closure expression in the instance
/// `inst`: a check gives a closure the def of its parent plus its
/// ordinal (`Ctx::closure_def`), and an instance's closures sit under
/// the instance's def.
pub(crate) fn instance_closure_map(p: &Program, tys: &impl TyIntern, t: Ty, template: LocalDefId, inst: DefId) -> Vec<(Ty, DefId)> {
    let mut cts: Vec<Ty> = Vec::new();
    collect_closures(tys, t, &mut cts);
    cts.into_iter()
        .filter_map(|ct| {
            let TyKind::Closure(d, _) = tys.kind(ct) else { return None };
            Some((ct, instance_closure_def(p, d, template, inst)?))
        })
        .collect()
}

/// The instance's def for the template closure `d` (see
/// `instance_closure_map`), through enclosing closures.
fn instance_closure_def(p: &Program, d: DefId, template: LocalDefId, inst: DefId) -> Option<DefId> {
    use crate::def::{DefKind, child_hash};
    let e = p.gx.closures.get(&d)?;
    let (old, new) = if e.parent == template {
        (p.def_id(template), inst)
    } else {
        let pd = *p.gx.closure_fns.get(&e.parent)?;
        (pd, instance_closure_def(p, pd, template, inst)?)
    };
    let ord = (0..4096u32).find(|&o| child_hash(old.hash, DefKind::Closure, "{closure}", o) == d.hash)?;
    Some(DefId { krate: new.krate, hash: child_hash(new.hash, DefKind::Closure, "{closure}", ord) })
}

/// True when `t` names a closure of a template body (a hidden type's
/// closure before its instance is known, `Checked::closure_map_pending`).
pub(crate) fn mentions_template_closure(p: &Program, tys: &impl TyIntern, t: Ty) -> bool {
    let mut cts = Vec::new();
    collect_closures(tys, t, &mut cts);
    cts.iter().any(|&ct| {
        matches!(tys.kind(ct), TyKind::Closure(d, _) if p.gx.closures.get(&d).is_some_and(|e| p.gx.fn_class.get(e.body_fn.0 as usize) == Some(&generics::FnClass::TemplateClosure)))
    })
}

/// The closure types in `t`, each once.
fn collect_closures(tys: &impl TyIntern, t: Ty, out: &mut Vec<Ty>) {
    let in_args = |a: ArgsId, out: &mut Vec<Ty>| {
        for g in tys.args(a) {
            if let GenericArg::Ty(e) = *g {
                collect_closures(tys, e, out);
            }
        }
    };
    match tys.kind(t) {
        TyKind::Closure(_, a) => {
            if !out.contains(&t) {
                out.push(t);
            }
            in_args(a, out);
        }
        TyKind::Ref(_, _, e) | TyKind::RawPtr(_, e) | TyKind::Slice(e) | TyKind::Array(e, _) => collect_closures(tys, e, out),
        TyKind::Tuple(ts) => tys.tys_of(ts).iter().for_each(|&e| collect_closures(tys, e, out)),
        TyKind::Adt(_, a) | TyKind::FnDef(_, a) | TyKind::Opaque(_, a) | TyKind::Projection(_, a) => in_args(a, out),
        _ => {}
    }
}

/// `g` with `map_closure_defs` applied when it is a type.
fn map_arg_closures(tys: &mut impl TyIntern, g: GenericArg, map: &[(Ty, DefId)]) -> GenericArg {
    match g {
        GenericArg::Ty(t) if !map.is_empty() => GenericArg::Ty(map_closure_defs(tys, t, map)),
        g => g,
    }
}

/// `t` with the def of each closure type `map` names replaced.
pub(crate) fn map_closure_defs(tys: &mut impl TyIntern, t: Ty, map: &[(Ty, DefId)]) -> Ty {
    let args = |tys: &mut _, a: ArgsId| {
        let old: Vec<GenericArg> = TyIntern::args(tys, a).to_vec();
        let new: Vec<GenericArg> = old
            .into_iter()
            .map(|g| match g {
                GenericArg::Ty(e) => GenericArg::Ty(map_closure_defs(tys, e, map)),
                g => g,
            })
            .collect();
        TyIntern::intern_args(tys, &new)
    };
    match tys.kind(t) {
        TyKind::Closure(d, a) => {
            let d = map.iter().find(|(x, _)| *x == t).map_or(d, |&(_, y)| y);
            let a = args(tys, a);
            tys.intern(TyKind::Closure(d, a))
        }
        TyKind::Ref(r, m, e) => {
            let e = map_closure_defs(tys, e, map);
            tys.intern(TyKind::Ref(r, m, e))
        }
        TyKind::Slice(e) => {
            let e = map_closure_defs(tys, e, map);
            tys.intern(TyKind::Slice(e))
        }
        TyKind::Array(e, n) => {
            let e = map_closure_defs(tys, e, map);
            tys.intern(TyKind::Array(e, n))
        }
        TyKind::Tuple(ts) => {
            let old: Vec<Ty> = tys.tys_of(ts).to_vec();
            let new: Vec<Ty> = old.into_iter().map(|e| map_closure_defs(tys, e, map)).collect();
            let ts = tys.intern_tys(&new);
            tys.intern(TyKind::Tuple(ts))
        }
        TyKind::Adt(d, a) => {
            let a = args(tys, a);
            tys.intern(TyKind::Adt(d, a))
        }
        _ => t,
    }
}

/// `reveal` over each type arg of `a`.
fn reveal_args(hidden: &FxMap<DefId, Ty>, tys: &mut impl TyIntern, a: ArgsId, depth: u32) -> ArgsId {
    let old: Vec<GenericArg> = tys.args(a).to_vec();
    let new: Vec<GenericArg> = old
        .into_iter()
        .map(|g| match g {
            GenericArg::Ty(e) => GenericArg::Ty(reveal(hidden, tys, e, depth + 1)),
            other => other,
        })
        .collect();
    tys.intern_args(&new)
}

/// True when a value of type `t` may own a value of type `inner` (is it,
/// or holds it by value; a reference owns nothing it points at).
pub(crate) fn ty_contains(tys: &impl TyIntern, t: Ty, inner: Ty) -> bool {
    if t == inner {
        return true;
    }
    let in_args = |a: ArgsId| tys.args(a).iter().any(|g| g.as_ty().is_some_and(|e| ty_contains(tys, e, inner)));
    match tys.kind(t) {
        TyKind::Slice(e) | TyKind::Array(e, _) => ty_contains(tys, e, inner),
        TyKind::Tuple(ts) => tys.tys_of(ts).iter().any(|&e| ty_contains(tys, e, inner)),
        TyKind::Adt(_, a) | TyKind::FnDef(_, a) | TyKind::Closure(_, a) | TyKind::Projection(_, a) => in_args(a),
        _ => false,
    }
}

/// The args of builtin iterator kind `k` that hold its inner iterators.
fn inner_iter_args(k: crate::ty::iter::IterKind) -> &'static [usize] {
    use crate::ty::iter::IterKind as K;
    match k {
        K::Map
        | K::Filter
        | K::FilterMap
        | K::MapWhile
        | K::Inspect
        | K::Enumerate
        | K::Take
        | K::Skip
        | K::Rev
        | K::Copied
        | K::Cloned
        | K::TakeWhile
        | K::SkipWhile
        | K::StepBy
        | K::Peekable
        | K::Flatten => &[0],
        K::Zip | K::Chain => &[0, 1],
        _ => &[],
    }
}

/// The types in `t` that sit in an inner iterator arg of a builtin
/// adapter and are no builtin iterator, each once (`canon_iter_args`).
fn bare_iter_args(tys: &impl TyIntern, t: Ty, out: &mut Vec<Ty>) {
    let in_args = |a: ArgsId, out: &mut Vec<Ty>| {
        for g in tys.args(a) {
            if let GenericArg::Ty(e) = *g {
                bare_iter_args(tys, e, out);
            }
        }
    };
    match tys.kind(t) {
        TyKind::Adt(_, a) => {
            if let Some((k, args)) = crate::ty::iter::iter_parts(tys, t) {
                for &i in inner_iter_args(k) {
                    if let Some(&s) = args.get(i)
                        && !crate::ty::iter::is_iter(tys, s)
                        && !out.contains(&s)
                    {
                        out.push(s);
                    }
                }
            }
            in_args(a, out);
        }
        TyKind::Closure(_, a) => in_args(a, out),
        TyKind::Ref(_, _, e) | TyKind::Slice(e) | TyKind::Array(e, _) => bare_iter_args(tys, e, out),
        TyKind::Tuple(ts) => tys.tys_of(ts).iter().for_each(|&e| bare_iter_args(tys, e, out)),
        _ => {}
    }
}

/// `t` with each inner iterator arg of a builtin adapter that `map`
/// names replaced (`bare_iter_args`), at every depth.
pub(super) fn replace_iter_args(tys: &mut impl TyIntern, t: Ty, map: &[(Ty, Ty)]) -> Ty {
    let args = |tys: &mut _, a: ArgsId, slots: &[usize]| {
        let old: Vec<GenericArg> = TyIntern::args(tys, a).to_vec();
        let new: Vec<GenericArg> = old
            .into_iter()
            .enumerate()
            .map(|(i, g)| match g {
                GenericArg::Ty(e) => {
                    let e = replace_iter_args(tys, e, map);
                    let e = if slots.contains(&i) { map.iter().find(|(x, _)| *x == e).map_or(e, |&(_, y)| y) } else { e };
                    GenericArg::Ty(e)
                }
                g => g,
            })
            .collect();
        TyIntern::intern_args(tys, &new)
    };
    match tys.kind(t) {
        TyKind::Adt(d, a) => {
            let slots = crate::ty::iter::iter_parts(tys, t).map_or(&[][..], |(k, _)| inner_iter_args(k));
            let a = args(tys, a, slots);
            tys.intern(TyKind::Adt(d, a))
        }
        TyKind::Closure(d, a) => {
            let a = args(tys, a, &[]);
            tys.intern(TyKind::Closure(d, a))
        }
        TyKind::Ref(r, m, e) => {
            let e = replace_iter_args(tys, e, map);
            tys.intern(TyKind::Ref(r, m, e))
        }
        TyKind::Slice(e) => {
            let e = replace_iter_args(tys, e, map);
            tys.intern(TyKind::Slice(e))
        }
        TyKind::Array(e, n) => {
            let e = replace_iter_args(tys, e, map);
            tys.intern(TyKind::Array(e, n))
        }
        TyKind::Tuple(ts) => {
            let old: Vec<Ty> = tys.tys_of(ts).to_vec();
            let new: Vec<Ty> = old.into_iter().map(|e| replace_iter_args(tys, e, map)).collect();
            let ts = tys.intern_tys(&new);
            tys.intern(TyKind::Tuple(ts))
        }
        _ => t,
    }
}

/// `ByNext<s, item>`, the wrapper a revealed check gives user iterator `s`
/// as an adapter's inner iterator (check/iter.rs `iter_method`), or None
/// when its item type is not known.
fn by_next_of(tys: &mut impl TyIntern, s: Ty, item: Ty) -> Option<Ty> {
    if item == ERROR || tys.flags(item).contains_projection() {
        return None;
    }
    Some(crate::ty::iter::mk_iter(tys, crate::ty::iter::IterKind::ByNext, &[s, item]))
}

/// `<s as Iterator>::Item` (`assoc_item` is the lang item), not normalized.
fn item_proj(assoc_item: DefId, tys: &mut impl TyIntern, s: Ty) -> Ty {
    let args = tys.intern_args(&[GenericArg::Ty(s)]);
    tys.intern(TyKind::Projection(AssocId(assoc_item), args))
}

/// True when `s` may be a user iterator: an Adt that is no builtin type.
fn maybe_user_iter(tys: &impl TyIntern, s: Ty) -> bool {
    tys.local_adt(s).is_some() && tys.builtin_of(s).is_none()
}

/// `reveal` for a type the program keeps (`FnInfo::ret`), normalized,
/// with each inner iterator of a builtin adapter typed as the defining
/// body's revealed check types it: a std collection iterator struct as
/// its inner builtin iterator (`collection_iter_inner`), and a user
/// iterator wrapped once as `ByNext`. A hidden type from the opaque view
/// or a template holds an opaque or a param there, so a plain reveal had
/// the bare type: `fn pass() -> impl Iterator { mk().map(f) }` with `mk`
/// returning `std::iter::from_fn(..)` has `Map<ByNext<FromFn<..>, T>, ..>`
/// in its body, and lowering found no item type in `Map<FromFn<..>, ..>`.
/// `Ctx::reveal_here` is the same inside a check.
pub(crate) fn reveal_global(p: &mut Program, hidden: &FxMap<DefId, Ty>, t: Ty) -> Ty {
    let t = reveal(hidden, &mut p.tys, t, 0);
    let t = generics::normalize_global(p, t);
    canon_global(p, t)
}

/// The inner iterator step of `reveal_global` alone. A user iterator
/// whose item type is not known yet (a closure no check registered)
/// stays bare.
fn canon_global(p: &mut Program, t: Ty) -> Ty {
    let mut bare = Vec::new();
    bare_iter_args(&p.tys, t, &mut bare);
    let mut map: Vec<(Ty, Ty)> = Vec::new();
    for s in bare {
        let inner = {
            let mut cx = crate::traits::solve::PureCx::new(&p.tys);
            p.collection_iter_inner(&mut cx.tys, s).and_then(|i| generics::snapshot_ty(&cx.tys, i))
        };
        let c = match inner {
            Some(o) => Some(generics::build_owned_ty(p, &o)),
            None if maybe_user_iter(&p.tys, s) => {
                // Mono's record of the Adt's `Iterator` impl has the item
                // type of an impl with a closure bound (`FromFn<F>`), which
                // the pure solver cannot normalize.
                let made = p.tys.local_adt(s).and_then(|a| p.gx.iter_methods.get(&a)).and_then(|m| m.3);
                let item = match made {
                    Some(item) => item,
                    None => {
                        let proj = item_proj(p.gx.lang.assoc_item, &mut p.tys, s);
                        generics::normalize_global(p, proj)
                    }
                };
                by_next_of(&mut p.tys, s, item)
            }
            None => None,
        };
        if let Some(c) = c {
            map.push((s, c));
        }
    }
    if map.is_empty() { t } else { replace_iter_args(&mut p.tys, t, &map) }
}

impl Ctx<'_> {
    /// `reveal` inside a check, with the inner iterators of builtin
    /// adapters typed as `reveal_global` types them. A callee's args can
    /// still be variables here (`pass(Cnt(0))`); body end fixes the slots
    /// they become (`build_closure_map`, `iter_arg_map`).
    pub(super) fn reveal_here(&mut self, t: Ty) -> Ty {
        let t = reveal(self.p.gx.opaque_hidden.as_map(), &mut self.tys, t, 0);
        let mut bare = Vec::new();
        bare_iter_args(&self.tys, t, &mut bare);
        let map = self.canon_iter_args(bare);
        if map.is_empty() { t } else { replace_iter_args(&mut self.tys, t, &map) }
    }

    /// The replacement of each bare inner iterator in `bare` (see
    /// `canon_global`). One whose item type is not known yet (a closure
    /// of a body no check registered: `FromFn<F>`'s item comes from
    /// `F`'s signature) makes the body check again, as for
    /// `build_closure_map`.
    fn canon_iter_args(&mut self, bare: Vec<Ty>) -> Vec<(Ty, Ty)> {
        let p = self.p;
        let mut map: Vec<(Ty, Ty)> = Vec::new();
        for s in bare {
            let c = match p.collection_iter_inner(&mut self.tys, s) {
                Some(i) => Some(i),
                None if maybe_user_iter(&self.tys, s) => {
                    let proj = item_proj(p.gx.lang.assoc_item, &mut self.tys, s);
                    let item = self.normalize_here(proj);
                    let c = by_next_of(&mut self.tys, s, item);
                    if c.is_none() {
                        self.out.closure_map_pending = true;
                    }
                    c
                }
                None => None,
            };
            if let Some(c) = c {
                map.push((s, c));
            }
        }
        map
    }
}

/// After mono: each `impl Trait` fn's revealed return type (`FnInfo::ret`,
/// which its C signature uses) made again as the fn's body and callers
/// see it: the closures of the generic opaques nested in its hidden type
/// are their instances' (`build_closure_map`), and the inner iterators
/// of builtin adapters are typed by `canon_global`, whose item types
/// need the closures every check registered. `reveal_plain` and
/// `fn_instance` reveal the return before those instances and closures
/// exist, so `fn pass() -> impl Iterator { def(5u8) }` kept `def`'s
/// template closure.
pub(crate) fn finish_opaque_rets(p: &mut Program) {
    let mut fns: Vec<(LocalDefId, Ty)> = p.gx.opaque_ret_of.iter().map(|(&f, &o)| (f, o)).collect();
    fns.sort_by_key(|(f, _)| f.0);
    for (f, opaque) in fns {
        let ret = p.fns[f.0 as usize].ret;
        let mut bare = Vec::new();
        bare_iter_args(&p.tys, ret, &mut bare);
        if bare.is_empty() && !mentions_template_closure(p, &p.tys, ret) {
            continue;
        }
        let hidden = std::mem::take(&mut p.gx.opaque_hidden);
        let r = reveal(hidden.as_map(), &mut p.tys, opaque, 0);
        let r = generics::normalize_global(p, r);
        let mut found: Vec<Ty> = Vec::new();
        collect_opaques_deep(hidden.as_map(), &mut p.tys, opaque, &mut found);
        let mut map: Vec<(Ty, DefId)> = Vec::new();
        // Innermost first, as in `build_closure_map`.
        for o in found.into_iter().rev() {
            let TyKind::Opaque(od, args) = p.tys.kind(o) else { continue };
            let Some(template) = p.gx.opaques.get(&od).and_then(|info| p.local_def(info.owner.hash)) else { continue };
            if p.gx.fn_templates.get(template.0 as usize).is_none_or(|t| t.is_none()) {
                continue;
            }
            let args: Vec<GenericArg> = p.tys.args(args).to_vec();
            let args: Vec<GenericArg> = args.into_iter().map(|g| map_arg_closures(&mut p.tys, g, &map)).collect();
            let Some(inst) = generics::instance_for(p, &p.tys, template, &args).filter(|&i| i != template) else { continue };
            let h = reveal(hidden.as_map(), &mut p.tys, o, 0);
            let h = generics::normalize_global(p, h);
            map.extend(instance_closure_map(p, &p.tys, h, template, p.def_id(inst)));
        }
        p.gx.opaque_hidden = hidden;
        let r = if map.is_empty() { r } else { map_closure_defs(&mut p.tys, r, &map) };
        p.fns[f.0 as usize].ret = canon_global(p, r);
    }
}

/// The opaque types in `t` and, at every depth, in their hidden types,
/// outermost first, each once. A plain `fn outer() -> impl Iterator`
/// whose body returns `inner::<u32>()` has the hidden type
/// `Opaque(inner, [u32])`: the closures that `outer()`'s value holds are
/// `inner::<u32>`'s, which only this walk finds (`callee_ret`,
/// `build_closure_map`).
fn collect_opaques_deep(hidden: &FxMap<DefId, Ty>, tys: &mut impl TyIntern, t: Ty, out: &mut Vec<Ty>) {
    collect_opaques(tys, t, out);
    let mut i = 0;
    while i < out.len() && i < 64 {
        if let TyKind::Opaque(od, a) = tys.kind(out[i])
            && let Some(&h) = hidden.get(&od)
        {
            let args: Vec<GenericArg> = tys.args(a).to_vec();
            let h = subst(tys, h, &args);
            collect_opaques(tys, h, out);
        }
        i += 1;
    }
}

/// The opaque types in `t`, outermost first, each once.
fn collect_opaques(tys: &impl TyIntern, t: Ty, out: &mut Vec<Ty>) {
    let mut in_args = |a: ArgsId, out: &mut Vec<Ty>| {
        for g in tys.args(a) {
            if let GenericArg::Ty(e) = g {
                collect_opaques(tys, *e, out);
            }
        }
    };
    match tys.kind(t) {
        TyKind::Opaque(_, a) => {
            if !out.contains(&t) {
                out.push(t);
            }
            in_args(a, out);
        }
        TyKind::Ref(_, _, e) | TyKind::RawPtr(_, e) | TyKind::Slice(e) | TyKind::Array(e, _) => collect_opaques(tys, e, out),
        TyKind::Tuple(ts) => {
            for &e in tys.tys_of(ts) {
                collect_opaques(tys, e, out);
            }
        }
        TyKind::Adt(_, a) | TyKind::FnDef(_, a) | TyKind::Closure(_, a) | TyKind::Projection(_, a) => in_args(a, out),
        _ => {}
    }
}

/// True when a type names an opaque type (the given one, or any).
pub(crate) fn mentions_opaque(tys: &impl TyIntern, t: Ty, which: Option<DefId>) -> bool {
    let in_args = |a: ArgsId| tys.args(a).iter().any(|g| g.as_ty().is_some_and(|e| mentions_opaque(tys, e, which)));
    match tys.kind(t) {
        TyKind::Opaque(d, a) => which.is_none_or(|w| w == d) || in_args(a),
        TyKind::Ref(_, _, e) | TyKind::RawPtr(_, e) | TyKind::Slice(e) | TyKind::Array(e, _) => mentions_opaque(tys, e, which),
        TyKind::Tuple(ts) => tys.tys_of(ts).iter().any(|&e| mentions_opaque(tys, e, which)),
        TyKind::Adt(_, a) | TyKind::FnDef(_, a) | TyKind::Closure(_, a) | TyKind::Projection(_, a) => in_args(a),
        TyKind::FnPtr(sig) => {
            let s = tys.sig(sig).value;
            tys.tys_of(s.inputs).iter().any(|&e| mentions_opaque(tys, e, which)) || mentions_opaque(tys, s.output, which)
        }
        _ => false,
    }
}

/// Reveals each plain fn's own `impl Trait` return for codegen: checks
/// the defining body, and on success sets `FnInfo::ret` to the hidden
/// type while callers keep the opaque (`GenTables::opaque_ret_of`). A
/// body with errors keeps its opaque; the driver's check reports them.
pub fn reveal_plain(p: &mut Program) {
    if p.gx.opaques.is_empty() {
        return;
    }
    let defining: Vec<LocalDefId> = (0..p.fns.len() as u32)
        .map(LocalDefId)
        .filter(|&f| {
            p.gx.fn_class.get(f.0 as usize).is_none_or(|c| *c == generics::FnClass::Plain)
                && matches!(p.tys.kind(p.fn_info(f).ret), TyKind::Opaque(od, _) if p.gx.opaques.get(&od).is_some_and(|o| o.owner == p.def_id(f)))
        })
        .collect();
    for f in defining {
        let (hidden, ok) = {
            let (checked, errs) = check_fn_view(p, f);
            let h = checked.opaque_hidden.first().map(|&(_, h)| h);
            (h.and_then(|h| crate::check::generics::snapshot_ty(&checked.tys, h)), errs.is_empty())
        };
        if let (Some(owned), true) = (hidden, ok) {
            let h = crate::check::generics::build_owned_ty(p, &owned);
            let opaque = p.fn_info(f).ret;
            if let TyKind::Opaque(od, _) = p.tys.kind(opaque) {
                p.gx.opaque_hidden.insert(od, h);
            }
            p.gx.opaque_ret_of.insert(f, opaque);
            p.fns[f.0 as usize].ret = h;
        }
    }
    reveal_plain_rets(p);
}

/// True when plain fn `f`'s own `impl Trait` return, revealed again from
/// its edited body (driver/patch.rs `reuse_kept`), is what `reveal_plain`
/// left in the program: the same hidden type, or no hidden type when the
/// body has errors now and had them before. A fresh collect would then
/// make the same program. The hidden type's closure types carry their
/// capture types; their signatures and kinds are the discovery output
/// `reuse_kept` compares.
pub(crate) fn hidden_stands(p: &mut Program, f: LocalDefId) -> bool {
    hidden_again(p, f) == HiddenAgain::Stands
}

/// What revealing plain fn `f`'s own `impl Trait` again gives
/// (`hidden_stands`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum HiddenAgain {
    /// The program's hidden type, or none before and after.
    Stands,
    /// Another hidden type, or one where the program has none.
    Moved,
    /// No hidden type: the body has errors now (it had none before).
    Errors,
}

/// `hidden_stands` with the reason it fails.
pub(crate) fn hidden_again(p: &mut Program, f: LocalDefId) -> HiddenAgain {
    let opaque = p.gx.opaque_ret_of.get(&f).copied().unwrap_or(p.fn_info(f).ret);
    let TyKind::Opaque(od, _) = p.tys.kind(opaque) else { return HiddenAgain::Moved };
    if p.gx.opaques.get(&od).is_none_or(|o| o.owner != p.def_id(f)) {
        return HiddenAgain::Moved;
    }
    let old = p.gx.opaque_ret_of.contains_key(&f).then(|| p.gx.opaque_hidden.get(&od).copied());
    // As `reveal_plain` runs in collect: before mono.
    let (counts, mono_done) = (p.gx.mono_counts, std::mem::replace(&mut p.gx.mono_done, false));
    let owned = {
        let (checked, errs) = check_fn_view(p, f);
        let h = checked.opaque_hidden.first().map(|&(_, h)| h);
        h.and_then(|h| crate::check::generics::snapshot_ty(&checked.tys, h)).filter(|_| errs.is_empty())
    };
    (p.gx.mono_counts, p.gx.mono_done) = (counts, mono_done);
    let new = owned.map(|o| Some(crate::check::generics::build_owned_ty(p, &o)));
    match (new == old, new) {
        (true, _) => HiddenAgain::Stands,
        (false, None) => HiddenAgain::Errors,
        (false, Some(_)) => HiddenAgain::Moved,
    }
}

/// Reveals the opaque types left in revealed plain fns' `FnInfo::ret` (a
/// hidden type that names another fn's opaque), so codegen sees concrete
/// types. Runs again once the template check has the hidden types of
/// generic fns.
pub fn reveal_plain_rets(p: &mut Program) {
    let mut revealed: Vec<LocalDefId> = p.gx.opaque_ret_of.keys().copied().collect();
    revealed.sort_by_key(|f| f.0);
    let hidden = p.gx.opaque_hidden.clone();
    for f in revealed {
        let r = p.fns[f.0 as usize].ret;
        p.fns[f.0 as usize].ret = reveal_global(p, hidden.as_map(), r);
    }
}
