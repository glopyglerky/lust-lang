// Rule zero against std items lust does not model (L0643).
//
// rustc's method probe walks the receiver's autoderef steps, and at each
// step probes by value, then `&`, then `&mut`. In one probe an inherent
// item beats a trait item, and two trait items are ambiguous (E0034).
// lust models only part of std, so a user trait's item could win in lust
// where std has an item of the same name that wins or ties in rustc
// (`v.spare_capacity_mut()` with a user trait method of that name: rustc
// calls Vec's). The other way, lust's builtin dispatch could answer where
// rustc picks the user trait first (`v.len()` with a by-value `fn
// len(self)` in a user trait for `Vec<u8>`: rustc's by-value probe finds
// the trait before Vec's `&self` len).
//
// `std_inherent.txt` (scripts/gen-std-inherent.py, from the rustdoc of std
// 1.99) lists what std gives each std type: its stable inherent items and
// the items of the std traits it implements, each with its receiver form.
// When an in-scope trait from outside std may apply to a std receiver
// step, `std_shadow_method` places both sides on rustc's probe order and
// says who wins; `std_shadow_path` does the same for `Type::item`, where
// every inherent item wins over every trait item.
use super::*;
use crate::ast::{ExprId, SelfKind};
use crate::def::DefId;
use crate::intern::FxMap;
use crate::traits::solve::Selection;
use std::sync::OnceLock;

/// Generated table (see the generator for the format): per std type its
/// items as receiver letter + name and its non-prelude std traits as
/// `@path`; per such trait a `#path` line of its items.
static TABLE: &str = include_str!("std_inherent.txt");

/// Item name to receiver letters.
type Items = FxMap<&'static str, Vec<u8>>;

/// The parsed table: std types (items, non-prelude traits) and the items
/// of those traits, by path.
#[derive(Default)]
struct Table {
    types: FxMap<&'static str, (Items, Vec<&'static str>)>,
    traits: FxMap<&'static str, Items>,
}

fn table() -> &'static Table {
    static T: OnceLock<Table> = OnceLock::new();
    T.get_or_init(|| {
        let mut t = Table::default();
        for line in TABLE.lines() {
            let mut words = line.split(' ');
            let Some(head) = words.next() else { continue };
            let (items, traits) = match head.strip_prefix('#') {
                Some(path) => (t.traits.entry(path).or_default(), None),
                None => {
                    let e = t.types.entry(head).or_default();
                    (&mut e.0, Some(&mut e.1))
                }
            };
            let mut traits = traits;
            for w in words {
                match (w.strip_prefix('@'), traits.as_deref_mut()) {
                    (Some(path), Some(ts)) => ts.push(path),
                    _ => {
                        if let Some((&letter, _)) = w.as_bytes().split_first() {
                            items.entry(&w[1..]).or_default().push(letter);
                        }
                    }
                }
            }
        }
        t
    })
}

/// What the method probe order says about a call whose receiver chain
/// has a std type and a user trait candidate (see `std_shadow_method`).
pub(super) enum StdShadow {
    /// No user trait item may apply, or std has no item of the name.
    None,
    /// rustc picks the user trait's method: std's item comes later.
    UserFirst,
    /// rustc picks std's inherent item: a user trait may not be picked.
    StdFirst,
    /// A std trait's item comes first or ties, or the user's receiver
    /// cannot be placed: only the trait probe may answer, and only with a
    /// std trait (it reports a tie it sees itself).
    Tie,
    /// The order cannot be told: report L0643.
    Reject,
}

impl Ctx<'_> {
    /// True when trait `d` is from outside std: the user's crates and the
    /// dependency adapters. Lang traits and std source traits are std.
    pub(super) fn is_user_trait(&self, d: DefId) -> bool {
        d.krate != crate::ty::BUILTIN_CRATE && !self.p.gx.index.trait_record(d).is_some_and(|r| self.std_span(r.span))
    }

    /// True for a span in std's own source (core, alloc, std): its calls
    /// were written against real std and its traits are std's.
    fn std_span(&self, span: Span) -> bool {
        use crate::ty::lang::ShippedCrate as K;
        self.p.std_source_span(span) || matches!(self.p.shipped_crate_at(span), Some(K::Core | K::Alloc | K::Std))
    }

    /// The table names of a std type: `u8`, `str`, `slice`, `Vec`, `Iter`
    /// (one name; an integer of unknown type gives both signs). Empty for
    /// every type that is not std's.
    fn std_type_names(&self, t: Ty) -> Vec<String> {
        let one = |s: &str| vec![s.to_string()];
        match self.tys.kind(t) {
            TyKind::Int(i) => one(i.name()),
            TyKind::Float(f) => one(f.name()),
            TyKind::Bool => one("bool"),
            TyKind::Char => one("char"),
            TyKind::Str => one("str"),
            TyKind::Slice(_) => one("slice"),
            TyKind::Array(..) => one("array"),
            // A reference or pointer step counts only over a std type:
            // std's impls for `&T` (`Iterator for &mut I`, `Display for
            // &T`) forward to `T`'s own, which a user type answers itself.
            TyKind::RawPtr(_, inner) if !self.std_type_names(inner).is_empty() => one("pointer"),
            TyKind::Ref(_, _, inner) if !self.std_type_names(inner).is_empty() => one("reference"),
            TyKind::FnPtr(_) => one("fn"),
            TyKind::Infer(v) if v.is_int() => vec!["i32".into(), "u32".into()],
            TyKind::Infer(v) if v.is_float() => vec!["f64".into(), "f32".into()],
            TyKind::Adt(..) => {
                let std = self.tys.builtin_of(t).is_some()
                    || self.tys.local_adt(t).is_some_and(|a| self.p.is_std_adt(a) || generics::is_std_adt(&self.tys, a));
                if !std {
                    return Vec::new();
                }
                // The printed path's last segment: `std::slice::Iter<u8>` is `Iter`.
                let shown = self.tn(t);
                let head = shown.split('<').next().unwrap_or("");
                one(head.rsplit("::").next().unwrap_or(head))
            }
            _ => Vec::new(),
        }
    }

    /// The receiver letters std gives `item` at type `t` (the union over
    /// its table names). Inference variables have no inherent candidates
    /// in rustc, so only their trait letters count.
    fn std_letters_at(&mut self, t: Ty, item: &str) -> Vec<u8> {
        let infer = matches!(self.tys.kind(t), TyKind::Infer(_));
        let tab = table();
        let mut out: Vec<u8> = Vec::new();
        for n in self.std_type_names(t) {
            let Some((items, traits)) = tab.types.get(n.as_str()) else { continue };
            out.extend(items.get(item).into_iter().flatten().filter(|l| !infer || l.is_ascii_uppercase()));
            for &path in traits {
                if let Some(ls) = tab.traits.get(path).and_then(|m| m.get(item))
                    && self.std_trait_may_be_in_scope(path)
                    && self.std_impl_may_apply(t, path)
                {
                    out.extend(ls);
                }
            }
        }
        out
    }

    /// True unless std trait `path` (not in the prelude) is surely out of
    /// scope at this body. rustc probes only the traits in scope where the
    /// call is (a where clause on a concrete type does not bring one in):
    /// a `use` in this module of a std item of its name (renamed, `as _`,
    /// or brought by a glob of a module that re-exports it), a glob of a
    /// std module in this module, a fn-local `use` in an enclosing block,
    /// or a std trait lust models under its name that is in scope here.
    /// A fn-local `use` lust does not record as a trait path (a glob, or
    /// a trait lust does not know) counts for the whole crate.
    fn std_trait_may_be_in_scope(&mut self, path: &str) -> bool {
        let p = self.p;
        let simple = path.rsplit("::").next().unwrap_or(path);
        let named = |s: &Sym| p.interner.get(*s) == simple;
        if self.std_trait_scopes.iter().flatten().any(|path| path.last().is_some_and(named)) || self.fn_local_std_use(simple) {
            return true;
        }
        // No resolver tables: the crate has no module-level `use`.
        if let Some(res) = &p.resolved {
            let module = self.module;
            if res.std_globs.iter().any(|(m, _)| *m == module)
                || res.std_aliases.iter().any(|(&(m, _, _), (_, path))| m == module && path.last().is_some_and(named))
            {
                return true;
            }
        }
        let modeled: Vec<DefId> = p.gx.trait_names.iter().filter(|(n, _)| named(n)).map(|(_, &d)| d).collect();
        modeled.into_iter().any(|d| !self.is_user_trait(d) && method::trait_in_scope_pub(self, d))
    }

    /// True when a fn-local `use` anywhere in the crate imports a std
    /// glob or a std item named `simple` (std, core, or alloc paths).
    fn fn_local_std_use(&self, simple: &str) -> bool {
        fn tree_hits(t: &crate::ast::UseTree, std: bool, simple: &str, interner: &crate::intern::Interner) -> bool {
            let std = std || t.prefix.first().is_some_and(|s| matches!(interner.get(s.name), "std" | "core" | "alloc"));
            match &t.kind {
                crate::ast::UseKind::Glob => std,
                crate::ast::UseKind::Simple { .. } => std && t.prefix.last().is_some_and(|s| interner.get(s.name) == simple),
                crate::ast::UseKind::Group(ts) => ts.iter().any(|c| tree_hits(c, std, simple, interner)),
            }
        }
        let p = self.p;
        p.ast.stmt_items.iter().any(|it| matches!(&it.kind, ItemKind::Use(u) if tree_hits(&u.tree, false, simple, &p.interner)))
    }

    /// False when std's impl of trait `path` for step type `t` surely
    /// does not apply, so the table's trait row must not count there:
    /// `Box`, `Rc`, and `Arc` forward a trait only when the pointee has it
    /// (`Rc<T>: Hash` needs `T: Hash`), `&T` and `&mut T` forward theirs,
    /// `Arc` has io `Read`, `Write`, and `Seek` only for `Arc<File>`, and
    /// `Rc` and a shared reference to a smart pointer have none. Only a
    /// pointee whose impls lust knows in full decides: a user type
    /// (selection) or a user trait object (its supertraits). Any other
    /// may hold, which can only give a tie (L0643).
    fn std_impl_may_apply(&mut self, t: Ty, path: &str) -> bool {
        const ALWAYS: &[&str] = &[
            "std::any::Any",
            "std::borrow::Borrow",
            "std::borrow::BorrowMut",
            "std::fmt::Pointer",
            "std::ops::Deref",
            "std::ops::DerefMut",
        ];
        if ALWAYS.contains(&path) {
            return true;
        }
        let io = matches!(path, "std::io::Read" | "std::io::Write" | "std::io::Seek" | "std::io::BufRead");
        let t = self.resolve(t);
        let smart = |cx: &Self, x: Ty| {
            matches!(cx.tys.builtin_of(x), Some(BuiltinTy::Boxed | BuiltinTy::Rc | BuiltinTy::Sync(crate::ty::sync::SyncTy::Arc)))
        };
        if let TyKind::Ref(_, m, inner) = self.tys.kind(t) {
            let inner = self.resolve(inner);
            if io && !m.is_mut() {
                return !smart(self, inner);
            }
            return self.std_impl_may_apply(inner, path);
        }
        if smart(self, t) {
            let Some(pointee) = self.tys.adt_first_ty_arg(t) else { return true };
            let pointee = self.resolve(pointee);
            return match self.tys.builtin_of(t) {
                Some(BuiltinTy::Rc) if io => false,
                Some(BuiltinTy::Sync(_)) if io => {
                    matches!(self.tys.kind(pointee), TyKind::Infer(_)) || self.std_type_names(pointee) == ["File"]
                }
                _ => self.std_impl_may_apply(pointee, path),
            };
        }
        // A leaf type: only one whose impls lust knows in full decides.
        // The std trait as lust's lang trait; any other may hold.
        let p = self.p;
        let lang = &p.gx.lang;
        let Some(d) = (match path {
            "std::hash::Hash" => Some(lang.hash),
            "std::hash::Hasher" => Some(lang.hasher),
            "std::fmt::Debug" => Some(lang.debug),
            "std::fmt::Display" => Some(lang.display),
            _ => None,
        }) else {
            return true;
        };
        if let Some(principal) = object::principal_of(&self.tys, t) {
            return !self.is_user_trait(principal.def) || object::principal_has_supertrait(p, principal.def, d);
        }
        let user_adt = matches!(self.tys.kind(t), TyKind::Adt(..)) && self.std_type_names(t).is_empty();
        !user_adt || self.std_trait_may_apply(d, t)
    }

    /// rustc's autoderef steps for a method probe from receiver type `rt`:
    /// references, `Box`, `Rc`, `Arc`, and guards, `Vec<T>` to `[T]`,
    /// `String` to `str`, then an array's unsizing to a slice.
    fn std_probe_steps(&mut self, rt: Ty) -> Vec<Ty> {
        let mut steps = Vec::new();
        let mut cur = self.resolve(rt);
        for _ in 0..16 {
            steps.push(cur);
            let next = match self.tys.kind(cur) {
                TyKind::Ref(_, _, inner) => Some(inner),
                _ => match self.tys.builtin_of(cur) {
                    Some(BuiltinTy::Vec) => self.tys.adt_first_ty_arg(cur).map(|el| self.tys.intern(TyKind::Slice(el))),
                    Some(BuiltinTy::String) => Some(self.tys.intern(TyKind::Str)),
                    _ => self.tys.smart_elem(cur),
                },
            };
            match next {
                Some(n) => cur = self.resolve(n),
                None => break,
            }
        }
        if let Some(&last) = steps.last()
            && let TyKind::Array(el, _) = self.tys.kind(last)
        {
            steps.push(self.tys.intern(TyKind::Slice(el)));
        }
        steps
    }

    /// True when trait `d` may hold with Self = `self_ty`: selection finds
    /// an impl or cannot rule one out (nested bounds are not proven, so
    /// this says yes more often than rustc). Nothing is kept.
    fn trait_may_apply(&mut self, d: DefId, self_ty: Ty) -> bool {
        let Some(n) = self.p.gx.index.trait_record(d).map(|r| r.generics.params.len()) else { return false };
        let snap = self.snapshot_vars();
        let obl_len = self.obl_len();
        let mut args = vec![GenericArg::Ty(self_ty)];
        for _ in 1..n {
            args.push(GenericArg::Ty(self.new_var(false)));
        }
        let args = self.tys.intern_args(&args);
        let sel = self.select_obligation(TraitRef { def: d, args, bindings: EMPTY_BINDINGS });
        self.rollback_vars(snap);
        self.obl_truncate(obl_len);
        !matches!(sel, Selection::NoImpl)
    }

    /// `trait_may_apply` for a std trait, through the nested bounds of the
    /// impl selection picks (three levels): a blanket impl holds only where
    /// its bounds may (`ToString` needs `Display`, `ToOwned` `Clone`,
    /// `IntoIterator` `Iterator`), so a user `fn to_string(&self)` on a type
    /// without Display is no tie. An open nested bound (`Into`'s `U:
    /// From<T>`) may hold.
    fn std_trait_may_apply(&mut self, d: DefId, self_ty: Ty) -> bool {
        // Core's `impl<T: 'static + ?Sized> Any for T`: the probe does not
        // check regions, so it may apply to every type.
        if d == self.p.gx.lang.any {
            return true;
        }

        let Some(n) = self.p.gx.index.trait_record(d).map(|r| r.generics.params.len()) else { return false };
        let snap = self.snapshot_vars();
        let obl_len = self.obl_len();
        let mut args = vec![GenericArg::Ty(self_ty)];
        for _ in 1..n {
            args.push(GenericArg::Ty(self.new_var(false)));
        }
        let args = self.tys.intern_args(&args);
        let ok = self.trait_ref_may_hold(TraitRef { def: d, args, bindings: EMPTY_BINDINGS }, 3);
        self.rollback_vars(snap);
        self.obl_truncate(obl_len);
        ok
    }

    fn trait_ref_may_hold(&mut self, tr: TraitRef, depth: u8) -> bool {
        // Core's `impl<I: Iterator + ?Sized> Iterator for &mut I`, which
        // lust's selection does not have.
        if tr.def == self.p.gx.lang.iterator
            && let Some(GenericArg::Ty(t)) = self.tys.args(tr.args).first().copied()
            && let TyKind::Ref(_, m, inner) = self.tys.kind(self.resolve(t))
            && m.is_mut()
        {
            let args = self.tys.intern_args(&[GenericArg::Ty(inner)]);
            return self.trait_ref_may_hold(TraitRef { args, ..tr }, depth);
        }
        match self.select_obligation(tr) {
            Selection::NoImpl => false,
            Selection::Impl { nested, .. } if depth > 0 => nested.into_iter().all(|pred| match pred {
                Predicate::Trait(b) if b.count == 0 => self.trait_ref_may_hold(b.value, depth - 1),
                _ => true,
            }),
            _ => true,
        }
    }

    /// The in-scope traits from outside std that declare a method or
    /// associated fn `name`, each with the receiver form of that item.
    fn user_traits_with(&mut self, name: Sym) -> Vec<(DefId, Option<SelfKind>)> {
        self.traits_with(name, true)
    }

    /// `user_traits_with` for the user's traits (`user`) or for std's.
    fn traits_with(&mut self, name: Sym, user: bool) -> Vec<(DefId, Option<SelfKind>)> {
        let p = self.p;
        let Some(ds) = p.gx.traits_by_method.get(&name) else { return Vec::new() };
        let mut out = Vec::new();
        for &d in ds {
            if self.is_user_trait(d) != user || !method::trait_in_scope_pub(self, d) {
                continue;
            }
            let Some(m) = p.gx.index.trait_record(d).and_then(|r| r.method_by_name(name)) else { continue };
            // A lang trait's method has its signature in `lang_method_sigs`.
            // `Any::type_id` has no lang signature (check/any.rs answers it).
            let kind = p.gx.trait_methods.get(&m).and_then(|t| t.self_kind).or_else(|| p.gx.lang_method_sigs.get(&m).and_then(|t| t.self_kind));
            let kind = kind.or((d == p.gx.lang.any).then_some(SelfKind::Ref)).or_else(|| method::hash_method_kind(self, d, name));
            out.push((d, kind));
        }
        // `Iterator`'s provided methods are builtins, not in its lang record.
        let iterator = p.gx.lang.iterator;
        if !user
            && let Some(kind) = iterator_provided_kind(p.interner.get(name))
            && method::trait_in_scope_pub(self, iterator)
        {
            out.push((iterator, Some(kind)));
        }
        out
    }

    /// The first probe (3 per autoderef step: by value, `&`, `&mut`) whose
    /// receiver type is `recv`, if any step reaches it. References compare
    /// by mutability and pointee, not by region.
    fn probe_of_receiver(&self, steps: &[Ty], recv: Ty) -> Option<usize> {
        let same = |a: Ty, b: Ty| match (self.tys.kind(a), self.tys.kind(b)) {
            (TyKind::Ref(_, m1, x), TyKind::Ref(_, m2, y)) => m1 == m2 && x == y,
            _ => a == b,
        };
        let autoref = |s: Ty, m: Mut| matches!(self.tys.kind(recv), TyKind::Ref(_, rm, inner) if rm == m && inner == s);
        for (i, &s) in steps.iter().enumerate() {
            if same(s, recv) {
                return Some(3 * i);
            }
            if autoref(s, Mut::Shared) {
                return Some(3 * i + 1);
            }
            if autoref(s, Mut::Mut) {
                return Some(3 * i + 2);
            }
        }
        None
    }

    /// The probe at which std's item of receiver letter `l` on step `j`
    /// matches: its receiver is the step, `&` it, or `&mut` it, placed by
    /// the same scan as the user side (a `&self` item of `T` matches at
    /// the by-value probe of a `&T` step before it, not of a `&mut T`).
    /// None for an item without `self`, which is no method; `Err` for a
    /// typed receiver (`self: Box<Self>`), which is not placed.
    fn std_probe(&mut self, steps: &[Ty], j: usize, l: u8) -> Result<Option<usize>, ()> {
        let s = steps[j];
        let recv = match l.to_ascii_lowercase() {
            b'v' => s,
            b'r' => self.tys.mk_ref(false, s),
            b'm' => self.tys.mk_ref(true, s),
            b't' => return Err(()),
            _ => return Ok(None),
        };
        Ok(self.probe_of_receiver(steps, recv))
    }

    /// The first probe at which trait `d`'s method of receiver form `kind`
    /// may apply over `steps`: Self is a step, `&` one, or `&mut` one (for
    /// a std trait, through its impl's bounds: `std_trait_may_apply`).
    /// None for an associated fn; a typed receiver is not placed, so it
    /// counts as the first probe.
    fn trait_probe_at(&mut self, steps: &[Ty], d: DefId, kind: Option<SelfKind>) -> Option<usize> {
        let mut first: Option<usize> = None;
        let std = !self.is_user_trait(d);
        for &s in steps {
            for self_ty in [s, self.tys.mk_ref(false, s), self.tys.mk_ref(true, s)] {
                let may = if std { self.std_trait_may_apply(d, self_ty) } else { self.trait_may_apply(d, self_ty) };
                if !may {
                    continue;
                }
                let recv = match kind? {
                    SelfKind::Value => self_ty,
                    SelfKind::Ref => self.tys.mk_ref(false, self_ty),
                    SelfKind::RefMut => self.tys.mk_ref(true, self_ty),
                    _ => return Some(0),
                };
                if let Some(at) = self.probe_of_receiver(steps, recv) {
                    first = Some(first.map_or(at, |f| f.min(at)));
                }
            }
        }
        first
    }

    /// True when an in-scope user trait's method `name`, or a user type's
    /// inherent method, may apply to call `e` (receiver type `rt`) at or
    /// before the probe whose receiver type is `std_recv`, a std item's:
    /// the user method probe must decide
    /// (`impl Tr for W` with `fn clone(&self)` comes before core's `Clone
    /// for &W` on a `&W` receiver, and ties with it on a `&&W` one).
    pub(super) fn user_trait_at_or_before(&mut self, e: ExprId, rt: Ty, name: Sym, std_recv: Ty) -> bool {
        let users = self.user_traits_with(name);
        let mut steps = self.std_probe_steps(rt);
        // Through a user `Deref`, rustc's step is the target place itself.
        if self.user_deref_call == Some(e) && steps.len() > 1 && matches!(self.tys.kind(steps[0]), TyKind::Ref(..)) {
            steps.remove(0);
        }
        let std_at = self.probe_of_receiver(&steps, std_recv);
        // A user type's inherent method at or before std's probe wins
        // (`S::clone(&self)` on `r: &S`, at the `&S` step's by-value probe).
        if self.inherent_probe(&steps, name).is_some_and(|i| std_at.is_none_or(|s| i <= s)) {
            return true;
        }
        users.into_iter().filter_map(|(d, kind)| self.trait_probe_at(&steps, d, kind)).any(|u| std_at.is_none_or(|s| u <= s))
    }

    /// Rule zero for `recv.name(..)` with receiver type `rt` (see the
    /// module comment): who wins in rustc's probe order. Instances skip
    /// it: the template check of the same body decided (rustc resolves a
    /// generic body's calls once).
    pub(super) fn std_shadow_method(&mut self, e: ExprId, rt: Ty, name: Sym, span: Span) -> StdShadow {
        if self.p.gx.instances.contains_key(&self.fid) || self.std_span(span) {
            return StdShadow::None;
        }
        let users = self.user_traits_with(name);
        let mut steps = self.std_probe_steps(rt);
        // Through a user `Deref`, rustc's step is the target place itself.
        if self.user_deref_call == Some(e) && steps.len() > 1 && matches!(self.tys.kind(steps[0]), TyKind::Ref(..)) {
            steps.remove(0);
        }
        if users.is_empty() {
            return self.std_before_inherent(&steps, name);
        }
        let item = self.name(name).to_string();
        // std's earliest inherent and trait probes.
        let (mut std_inh, mut std_tr): (Option<usize>, Option<usize>) = (None, None);
        let mut std_typed = false;
        for j in 0..steps.len() {
            for l in self.std_letters_at(steps[j], &item) {
                let Ok(at) = self.std_probe(&steps, j, l) else {
                    std_typed = true;
                    continue;
                };
                let Some(at) = at else { continue };
                let slot = if l.is_ascii_uppercase() { &mut std_tr } else { &mut std_inh };
                if slot.is_none_or(|s| at < s) {
                    *slot = Some(at);
                }
            }
        }
        // std's traits place like the user side below: their blanket and
        // derived impls apply to every step type, user Adts and references
        // included (core's `Into` for every Sized type, `Clone for &T`, a
        // derived `PartialEq`), which the table of std types leaves out.
        // A parameter's bounds are inherent candidates in rustc, which win
        // over every trait in scope at their step (`v.into()` on `I:
        // IntoIter`): the generic method search places those steps.
        let generic = steps.iter().any(|&s| {
            let mut t = s;
            while let TyKind::Ref(_, _, inner) = self.tys.kind(t) {
                t = self.resolve(inner);
            }
            matches!(self.tys.kind(t), TyKind::Param(_) | TyKind::Projection(..) | TyKind::Opaque(..) | TyKind::Dyn(..) | TyKind::Infer(_))
        });
        let mut std_placed: Option<usize> = None;
        for (d, kind) in if generic { Vec::new() } else { self.traits_with(name, false) } {
            if let Some(at) = self.trait_probe_at(&steps, d, kind) {
                std_tr = Some(std_tr.map_or(at, |t| t.min(at)));
                std_placed = Some(std_placed.map_or(at, |t| t.min(at)));
            }
        }
        if std_inh.is_none() && std_tr.is_none() && !std_typed {
            return StdShadow::None;
        }
        // The user side's earliest probe. A candidate's receiver is
        // `form(Self)`, and a probe takes it only when that is a step, `&`
        // a step, or `&mut` a step, so Self is a step or a reference to one.
        // A typed receiver (`self: Rc<Self>`) is not placed: `unplaced`.
        let mut user: Option<usize> = None;
        let mut unplaced = false;
        for &(d, kind) in &users {
            if kind.is_none() {
                continue; // An associated fn is no method candidate.
            }
            for j in 0..steps.len() {
                let s = steps[j];
                for self_ty in [s, self.tys.mk_ref(false, s), self.tys.mk_ref(true, s)] {
                    if !self.trait_may_apply(d, self_ty) {
                        continue;
                    }
                    let recv = match kind {
                        Some(SelfKind::Value) => self_ty,
                        Some(SelfKind::Ref) => self.tys.mk_ref(false, self_ty),
                        Some(SelfKind::RefMut) => self.tys.mk_ref(true, self_ty),
                        _ => {
                            unplaced = true;
                            continue;
                        }
                    };
                    if let Some(at) = self.probe_of_receiver(&steps, recv) {
                        user = Some(user.map_or(at, |u| u.min(at)));
                    }
                }
            }
        }
        let Some(user) = user.filter(|_| !unplaced) else {
            return match (unplaced, std_typed || std_inh.is_some()) {
                (false, _) => StdShadow::None,
                (true, true) => StdShadow::Reject,
                (true, false) => StdShadow::Tie,
            };
        };
        // A typed std receiver is not placed: the order is unknown.
        if std_typed {
            return StdShadow::Reject;
        }
        // In one probe an inherent item wins; a std trait's item first or
        // at the user's probe is the trait probe's to decide. A std trait
        // placed by its impls (`std_placed`) strictly first wins, so the
        // user trait is skipped.
        match (std_inh, std_tr) {
            (Some(i), t) if i <= user && t.is_none_or(|t| i <= t) => StdShadow::StdFirst,
            (_, Some(t)) if t == user || (t < user && std_placed.is_none_or(|p| p > t)) => StdShadow::Tie,
            (_, Some(_)) if std_placed.is_some_and(|p| p < user) => StdShadow::StdFirst,
            _ => StdShadow::UserFirst,
        }
    }

    /// The first probe over `steps` at which a user type's inherent
    /// method `name` applies (its receiver is the step, `&` it, or `&mut`
    /// it), if any.
    fn inherent_probe(&mut self, steps: &[Ty], name: Sym) -> Option<usize> {
        let p = self.p;
        let mut inh: Option<usize> = None;
        for &s in steps {
            let Some((adt, _)) = self.tys.adt_parts(s) else { continue };
            if !self.std_type_names(s).is_empty() {
                continue;
            }
            let Some(fs) = p.gx.inherent_methods.get(&(adt, name)) else { continue };
            for &f in fs {
                let recv = match p.fn_info(f).self_kind {
                    Some(SelfKind::Value) => s,
                    Some(SelfKind::Ref) => self.tys.mk_ref(false, s),
                    Some(SelfKind::RefMut) => self.tys.mk_ref(true, s),
                    _ => continue,
                };
                if let Some(at) = self.probe_of_receiver(steps, recv) {
                    inh = Some(inh.map_or(at, |i| i.min(at)));
                }
            }
        }
        inh
    }

    /// With no user trait method of the name: a user type's inherent
    /// method `name` found at a later autoderef step loses to a std item
    /// that applies at an earlier probe (`(&&v).to_owned()` is core's
    /// `ToOwned for &S` at the `&&S` step, not `S::to_owned`). Std's side
    /// is a std type's inherent item (the table) or a std trait in scope
    /// placed by its impls (selection); lust's builtin dispatch does not
    /// give that pick here, so the call is L0643. In one probe the
    /// inherent method wins, as in rustc.
    fn std_before_inherent(&mut self, steps: &[Ty], name: Sym) -> StdShadow {
        let Some(inh) = self.inherent_probe(steps, name) else { return StdShadow::None };
        if inh == 0 {
            return StdShadow::None;
        }
        // `clone` on an `Rc` or `Arc` receiver (a value, or one shared
        // reference to it) is the pointer's own Clone at the first step,
        // which the builtin clone dispatch gives: a `T::clone` at a deref
        // step loses (ts-rust `Rc<ProjectCollection>`).
        if self.name(name) == "clone"
            && let Some(&s) = steps.first()
        {
            let s = match self.tys.kind(s) {
                TyKind::Ref(_, Mut::Shared, inner) => self.resolve(inner),
                _ => s,
            };
            if self.tys.rc_elem(s).is_some() {
                return StdShadow::None;
            }
        }
        let item = self.name(name).to_string();
        let mut std_at: Option<usize> = None;
        for j in 0..steps.len() {
            for l in self.std_letters_at(steps[j], &item) {
                if l.is_ascii_lowercase()
                    && let Ok(Some(at)) = self.std_probe(steps, j, l)
                {
                    std_at = Some(std_at.map_or(at, |s| s.min(at)));
                }
            }
        }
        let generic = steps.iter().any(|&s| {
            let mut t = s;
            while let TyKind::Ref(_, _, inner) = self.tys.kind(t) {
                t = self.resolve(inner);
            }
            matches!(self.tys.kind(t), TyKind::Param(_) | TyKind::Projection(..) | TyKind::Opaque(..) | TyKind::Dyn(..) | TyKind::Infer(_))
        });
        if !generic {
            for (d, kind) in self.traits_with(name, false) {
                if let Some(at) = self.trait_probe_at(steps, d, kind) {
                    std_at = Some(std_at.map_or(at, |s| s.min(at)));
                }
            }
        }
        if std_at.is_some_and(|s| s < inh) { StdShadow::Reject } else { StdShadow::None }
    }

    /// `std_shadow_method` at a method call `e` (receiver type `rt`, `base`
    /// after autoderef), acted on before the builtin dispatch: Some when
    /// the call is answered here (the user trait's pick, or L0643).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn std_shadow_call(
        &mut self,
        e: ExprId,
        rt: Ty,
        base: Ty,
        name: Sym,
        span: Span,
        args: &[ExprId],
        explicit: &[Option<GenericArg>],
        expected: Option<Ty>,
    ) -> Option<Ty> {
        let tie = match self.std_shadow_method(e, rt, name, span) {
            StdShadow::None => return None,
            StdShadow::StdFirst => {
                self.std_first.push(e);
                return None;
            }
            StdShadow::Reject => {
                self.report_std_shadow(base, name, false, span);
                for &a in args {
                    self.expr(a, None);
                }
                return Some(ERROR);
            }
            StdShadow::UserFirst => false,
            StdShadow::Tie => true,
        };
        // The builtin dispatch would answer with std's item, so the trait
        // probe runs first, alone. On a tie only a std trait's pick stands.
        self.last_pick = None;
        let hit = method::generic_method_search(self, e, rt, name, span, args, explicit, expected);
        match hit {
            method::Probe::Failed => return Some(ERROR),
            method::Probe::Hit(ret) if !tie || self.last_pick.is_some_and(|(pe, d)| pe == e && !self.is_user_trait(d)) => return Some(ret),
            _ => {}
        }
        self.report_std_shadow(base, name, false, span);
        if matches!(hit, method::Probe::Miss) {
            for &a in args {
                self.expr(a, None);
            }
        }
        Some(ERROR)
    }

    /// Rule zero for a type-relative path `Type::name` with no written
    /// trait, at Self type `base`: every std item of the name on a std
    /// type wins over (inherent) or ties with (trait) a user trait's item,
    /// so a user trait item that may apply there is L0643. True when
    /// reported. `consts` also counts user traits' associated consts.
    pub(super) fn std_shadow_path(&mut self, base: Ty, name: Sym, consts: bool, span: Span) -> bool {
        if self.p.gx.instances.contains_key(&self.fid) || self.std_span(span) {
            return false;
        }
        let base = self.resolve(base);
        let item = self.name(name).to_string();
        if self.std_letters_at(base, &item).is_empty() {
            return false;
        }
        let p = self.p;
        let mut users: Vec<DefId> = self.user_traits_with(name).into_iter().map(|(d, _)| d).collect();
        if consts {
            for &assoc in p.gx.trait_assoc_consts.keys() {
                let Some(t) = p.gx.index.trait_of_assoc(assoc) else { continue };
                if p.gx.index.trait_record(t).is_some_and(|r| r.assoc_items.iter().any(|&(d, n, _)| d == assoc && n == name))
                    && !users.contains(&t)
                    && self.is_user_trait(t)
                    && method::trait_in_scope_pub(self, t)
                {
                    users.push(t);
                }
            }
        }
        if users.into_iter().any(|d| self.trait_may_apply(d, base)) {
            self.report_std_shadow(base, name, true, span);
            return true;
        }
        false
    }

    /// The L0643 diagnostic: std's item `item` on `ty` and a user trait's
    /// item of that name, where lust cannot give rustc's pick.
    pub(super) fn report_std_shadow(&mut self, ty: Ty, item: Sym, path: bool, span: Span) {
        let (tn, item) = (self.tn(ty), self.name(item).to_string());
        let call = if path { format!("`<Type as Trait>::{item}`") } else { format!("`Trait::{item}(x)`") };
        self.err(
            "L0643",
            span,
            format!("std's `{item}` for `{tn}` may win over or tie with a trait item of that name here, and lust cannot decide it yet"),
            format!("Name the trait in the call, as in {call}, or rename the trait item."),
        );
    }
}

/// The receiver form of `Iterator`'s stable provided method `name`, if it
/// is one (rustc 1.99).
fn iterator_provided_kind(name: &str) -> Option<SelfKind> {
    const BY_MUT: &[&str] = &["nth", "by_ref", "any", "all", "find", "find_map", "position", "rposition", "try_fold", "try_for_each"];
    const BY_VALUE: &[&str] = &[
        "count", "last", "step_by", "chain", "zip", "map", "for_each", "filter", "filter_map", "enumerate", "peekable", "skip_while",
        "take_while", "map_while", "skip", "take", "scan", "flat_map", "flatten", "fuse", "inspect", "collect", "partition", "fold",
        "reduce", "max", "min", "max_by_key", "max_by", "min_by_key", "min_by", "rev", "unzip", "copied", "cloned", "cycle", "sum",
        "product", "cmp", "partial_cmp", "eq", "ne", "lt", "le", "gt", "ge", "is_sorted", "is_sorted_by", "is_sorted_by_key",
    ];
    if BY_MUT.contains(&name) {
        Some(SelfKind::RefMut)
    } else if BY_VALUE.contains(&name) {
        Some(SelfKind::Value)
    } else {
        None
    }
}
