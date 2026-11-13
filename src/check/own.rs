// Owning pointers to unsized values (w-unsized): `Box<str>`, `Rc<str>`,
// `Arc<str>`, `Rc<[T]>`, `Arc<[T]>`, and `Arc<dyn Tr>`, next to the
// older `Box<[T]>`, `Box<dyn Tr>`, and `Rc<dyn Tr>`. They are the plain
// `Box`, `Rc`, and `Arc` Adts with an unsized argument; codegen gives
// them a fat C form (cgen/own.rs).
//
// This file holds the std `From` impls that build one (`From<&str>`,
// `From<String>`, `From<Vec<T>>`, ..., lowered to `Builtin::OwnFrom`)
// and the rules for which pointee each pointer may have.
use super::*;

/// The owning pointer kind of a `CTy::Own` and of `Builtin::OwnFrom`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Owner {
    Box,
    Rc,
    Arc,
}

impl Owner {
    /// The mangle token of `CTy::Own` with this owner.
    pub fn code(self) -> &'static str {
        match self {
            Owner::Box => "OwB",
            Owner::Rc => "OwR",
            Owner::Arc => "OwA",
        }
    }
}

/// The owner and pointee of `t` when it is a `Box`, `Rc`, or `Arc` of
/// an unsized type (`str`, `[T]`, `dyn Tr`, `RefCell<dyn Tr>`, or
/// `Mutex<dyn Tr>`).
pub fn own_parts(tys: &impl TyIntern, t: Ty) -> Option<(Owner, Ty)> {
    let k = match tys.builtin_of(t)? {
        BuiltinTy::Boxed => Owner::Box,
        BuiltinTy::Rc => Owner::Rc,
        BuiltinTy::Sync(crate::ty::sync::SyncTy::Arc) => Owner::Arc,
        _ => return None,
    };
    let e = tys.adt_first_ty_arg(t)?;
    (matches!(tys.kind(e), TyKind::Str | TyKind::Slice(_) | TyKind::Dyn(..)) || dyn_wrapped_elem(tys, e).is_some()).then_some((k, e))
}

/// The `str` or `[T]` pointee of an `Rc` or `Arc` (`Rc<str>`, `Arc<[T]>`).
pub fn shared_unsized_elem(tys: &impl TyIntern, t: Ty) -> Option<Ty> {
    match own_parts(tys, t)? {
        (Owner::Rc | Owner::Arc, e) if matches!(tys.kind(e), TyKind::Str | TyKind::Slice(_)) => Some(e),
        _ => None,
    }
}

/// The owner of a `From` impl that builds `dst` from `src` (std's impls
/// for `Box<str>`, `Rc<str>`, `Arc<str>`, `Box<[T]>`, `Rc<[T]>`, and
/// `Arc<[T]>`), if std has one. Cloning sources need `T: Clone`.
pub fn own_from(p: &Program, tys: &impl TyIntern, dst: Ty, src: Ty) -> Option<Owner> {
    let (k, e) = own_parts(tys, dst)?;
    // `&str` and `&[T]`, shared or mutable (std has both since 1.84).
    let inner = |t: Ty| match tys.kind(t) {
        TyKind::Ref(_, _, i) => Some(i),
        _ => None,
    };
    let ok = match tys.kind(e) {
        TyKind::Str => {
            inner(src).is_some_and(|i| tys.kind(i) == TyKind::Str)
                || tys.is_string_ty(src)
                || (k != Owner::Box && own_parts(tys, src).is_some_and(|(sk, se)| sk == Owner::Box && tys.kind(se) == TyKind::Str))
        }
        TyKind::Slice(el) => {
            let src_elem = match tys.kind(src) {
                TyKind::Array(x, _) => Some(x),
                _ if k == Owner::Box => None,
                _ => tys.vec_elem(src).or_else(|| tys.boxed_slice_elem(src)),
            };
            src_elem == Some(el) || inner(src).is_some_and(|i| tys.kind(i) == TyKind::Slice(el) && has_clone(p, tys, el))
        }
        _ => false,
    };
    ok.then_some(k)
}

/// The owner of alloc's `impl<T: ?Sized> From<Box<T>> for Rc<T>` (and
/// `Arc<T>`) when it builds `dst` from `src` and `T` is sized or a plain
/// `dyn` (`str` and `[T]` are `own_from`'s; a `RefCell<dyn Tr>` or a
/// user type with an unsized tail is not built). A `dyn` pointee may
/// differ in its object lifetime only (`'static` against an inferred
/// one): lowering moves the box through a `Box` of the target's pointee
/// first, so the borrow checker relates the two (lower/iter.rs `conv`).
pub fn own_from_box(tys: &impl TyIntern, dst: Ty, src: Ty) -> Option<Owner> {
    let k = match tys.builtin_of(dst)? {
        BuiltinTy::Rc => Owner::Rc,
        BuiltinTy::Sync(crate::ty::sync::SyncTy::Arc) => Owner::Arc,
        _ => return None,
    };
    let (e, b) = (tys.rc_elem(dst)?, tys.boxed_elem(src)?);
    let ok = match (tys.kind(e), tys.kind(b)) {
        (TyKind::Dyn(x, _), TyKind::Dyn(y, _)) => x == y,
        _ => e == b && crate::check::collect::is_sized_ty(tys, e),
    };
    ok.then_some(k)
}

impl<'p> Ctx<'p> {
    /// The signature of `Box::from(x)`, `Rc::from(x)`, or `Arc::from(x)`
    /// (owner `k`). The target is the written Self (`Rc::<str>::from`),
    /// else the expected type: rustc finds a `From` impl for every
    /// pointee (`From<T> for Rc<T>` always applies), so a call with
    /// neither is E0283. A target std converts to (check/convert.rs
    /// `std_from`) makes the call `Builtin::ConvInto`.
    pub(super) fn own_from_sig(
        &mut self,
        k: Owner,
        callee: ExprId,
        e: ExprId,
        args: &[ExprId],
        want: Option<Ty>,
        span: Span,
    ) -> (Vec<Ty>, Ty) {
        let at = if args.len() == 1 { self.expr(args[0], None) } else { ERROR };
        let at = self.deep(at);
        let builtin = match k {
            Owner::Box => BuiltinTy::Boxed,
            Owner::Rc => BuiltinTy::Rc,
            Owner::Arc => BuiltinTy::Sync(crate::ty::sync::SyncTy::Arc),
        };
        let dst = want.map(|t| self.deep(t)).filter(|&t| self.tys.builtin_of(t) == Some(builtin));
        let Some(dst) = dst.filter(|&t| !self.tys.flags(t).contains_infer()) else {
            // No target yet: rustc keeps `Box<?U>: From<A>` pending until
            // a later coercion fixes `?U` (`Box::leak(Box::from(s))` into
            // `&str`). The pick waits for finish (`deferred_own_from`).
            if args.len() == 1 && !self.has_error_kind(at) {
                let u = self.new_var(false);
                let open = match k {
                    Owner::Box => self.tys.mk_boxed(u),
                    Owner::Rc => self.tys.mk_rc(u),
                    Owner::Arc => crate::ty::sync::sync_ty(&mut self.tys, crate::ty::sync::SyncTy::Arc, &[u]),
                };
                if let Some(w) = dst
                    && !self.unify(w, open)
                {
                    self.mismatch(span, w, open);
                }
                self.deferred_own_from.push((callee, e, open, at, span));
                return (vec![at], open);
            }
            if !self.has_error_kind(at) {
                self.err(
                    "L0610",
                    span,
                    "type annotations needed: cannot infer the type this `from` builds",
                    "Write the target type, for example `let s: Rc<str> = Rc::from(x);`.",
                );
            }
            return (vec![at], ERROR);
        };
        if self.has_error_kind(at) {
            return (vec![at], dst);
        }
        // A literal's integer or float type comes from the target
        // (`Rc::<[u8]>::from(vec![1, 2])`), as the one std impl fixes it,
        // and so does an open element (`Rc::<[u8]>::from(vec![])`).
        if self.std_from_arg(dst, at).is_none() {
            self.unify_open_elem(dst, at);
            self.unify_open_box(dst, at);
        }
        let lit = if self.std_from_arg(dst, at).is_none() { self.std_from_literal_src(dst, at) } else { None };
        let at = self.deep(at);
        match lit.or_else(|| self.std_from_arg(dst, at)) {
            Some(a) => {
                self.set_res(callee, Res::Builtin(Builtin::ConvInto));
                self.set_res(e, Res::Builtin(Builtin::ConvInto));
                (vec![a], dst)
            }
            None => {
                // A `dyn` pointee (`Box<dyn Error>` from an error
                // value, F3) has no builtin build here: the call is
                // the library `From` impl, selected at body end as
                // `.into()` on the same value is.
                if let Some((_, pt)) = own_parts(&self.tys, dst)
                    && matches!(self.tys.kind(pt), TyKind::Dyn(..))
                {
                    let targs = self.tys.intern_args(&[GenericArg::Ty(dst), GenericArg::Ty(at)]);
                    let trait_ref = TraitRef { def: self.p.gx.lang.from, args: targs, bindings: EMPTY_BINDINGS };
                    self.obl.push_trait_at(trait_ref, span, 0);
                    super::method::defer_path_call(self, e, trait_ref, self.p.interner.intern("from"), vec![]);
                    self.record_pick(e, self.p.gx.lang.from, 0, ReceiverForm::Value);
                    return (vec![at], dst);
                }
                let (dn, an) = (self.tn(dst), self.tn(at));
                self.err(
                    "L0612",
                    span,
                    format!("`{dn}` has no `From<{an}>` impl that lust supports"),
                    "Convert from a `&str`, a `String`, a slice, a `Vec`, or an array.",
                );
                (vec![at], ERROR)
            }
        }
    }
}

/// std's impls for the owning pointers to `str` and `[T]`, as synthetic
/// impl records for trait selection (`T: Into<Rc<str>>` bounds, derives
/// over such fields): the `From` impls `own_from` answers, alloc's
/// `From<Box<T>>` for `Rc<T>` and `Arc<T>` (`own_from_box`) and
/// `From<T> for Arc<T>`, `Clone` of
/// `Box<str>` and `Box<[T]>` (T: Clone), and `Default` of each but
/// `Box<[T]>` (the generic `Box<T>`, `Rc<T>`, and `Arc<T>` records need
/// a `T: Default`, which `str` and `[T]` are not).
pub fn add_own_impls(p: &mut Program, ordinal: &mut u32, span: Span) {
    use crate::check::generics::insert_synthetic;
    use crate::traits::index::Synthetic;
    let lang = p.gx.lang;
    let g0 = Generics::default();
    let p0 = p.tys.intern(TyKind::Param(ParamId(0)));
    let one = Generics {
        parent: None,
        params: vec![ParamDef { name: p.interner.intern("T"), kind: ParamKind::Type, span }],
        predicates: Vec::new(),
    };
    let mut cloned = one.clone();
    let a = p.tys.intern_args(&[GenericArg::Ty(p0)]);
    cloned.predicates = vec![Predicate::Trait(Binder::plain(TraitRef { def: lang.clone, args: a, bindings: EMPTY_BINDINGS }))];
    let slice_p0 = p.tys.intern(TyKind::Slice(p0));
    let (ref_str, mut_str) = (p.tys.mk_ref(false, STR), p.tys.mk_ref(true, STR));
    let (ref_slice, mut_slice) = (p.tys.mk_ref(false, slice_p0), p.tys.mk_ref(true, slice_p0));
    let vec_p0 = p.tys.mk_vec(p0);
    let mk = |p: &mut Program, k: Owner, e: Ty| match k {
        Owner::Box => p.tys.mk_boxed(e),
        Owner::Rc => p.tys.mk_rc(e),
        Owner::Arc => crate::ty::sync::sync_ty(&mut p.tys, crate::ty::sync::SyncTy::Arc, &[e]),
    };
    let box_str = mk(p, Owner::Box, STR);
    let box_slice = mk(p, Owner::Box, slice_p0);
    for k in [Owner::Box, Owner::Rc, Owner::Arc] {
        let (s, sl) = (mk(p, k, STR), mk(p, k, slice_p0));
        for src in [ref_str, mut_str, STRING] {
            insert_synthetic(p, ordinal, lang.from, s, &[src], g0.clone(), Vec::new(), Synthetic::Lang, false, span);
        }
        for src in [ref_slice, mut_slice] {
            insert_synthetic(p, ordinal, lang.from, sl, &[src], cloned.clone(), Vec::new(), Synthetic::Lang, false, span);
        }
        if k != Owner::Box {
            insert_synthetic(p, ordinal, lang.from, sl, &[vec_p0], one.clone(), Vec::new(), Synthetic::Lang, false, span);
            // alloc's `impl<T: ?Sized> From<Box<T>>` (`Box<str>`,
            // `Box<[T]>`, `Box<dyn Tr>`, a sized box: `own_from_box`).
            let (ptr, boxed) = (mk(p, k, p0), mk(p, Owner::Box, p0));
            insert_synthetic(p, ordinal, lang.from, ptr, &[boxed], one.clone(), Vec::new(), Synthetic::Lang, false, span);
        }
        // `impl<T> From<T> for Arc<T>` (`Rc<T>`'s is generics.rs's).
        if k == Owner::Arc {
            let ptr = mk(p, k, p0);
            insert_synthetic(p, ordinal, lang.from, ptr, &[p0], one.clone(), Vec::new(), Synthetic::Lang, false, span);
        }
        insert_synthetic(p, ordinal, lang.default, s, &[], g0.clone(), Vec::new(), Synthetic::Lang, false, span);
        // `Box<[T]>`'s Default is generics.rs's (fx-defaults).
        if k != Owner::Box {
            insert_synthetic(p, ordinal, lang.default, sl, &[], one.clone(), Vec::new(), Synthetic::Lang, false, span);
        }
    }
    insert_synthetic(p, ordinal, lang.from, STRING, &[box_str], g0.clone(), Vec::new(), Synthetic::Lang, false, span);
    // `str: Debug`, and alloc's `impl<T: Display + ?Sized> Display` for
    // `Box<T>` and `Rc<T>` (`Arc<T>`'s is check/sync.rs's), so a
    // `T: Debug` or `T: Display` bound holds at `Box<str>`, `Rc<str>`.
    insert_synthetic(p, ordinal, lang.debug, STR, &[], g0.clone(), Vec::new(), Synthetic::Lang, false, span);
    let a = p.tys.intern_args(&[GenericArg::Ty(p0)]);
    let mut shown = one.clone();
    shown.predicates = vec![Predicate::Trait(Binder::plain(TraitRef { def: lang.display, args: a, bindings: EMPTY_BINDINGS }))];
    for t in [mk(p, Owner::Box, p0), mk(p, Owner::Rc, p0)] {
        insert_synthetic(p, ordinal, lang.display, t, &[], shown.clone(), Vec::new(), Synthetic::Lang, false, span);
    }
    insert_synthetic(p, ordinal, lang.clone, box_str, &[], g0, Vec::new(), Synthetic::Lang, false, span);
    insert_synthetic(p, ordinal, lang.clone, box_slice, &[], cloned, Vec::new(), Synthetic::Lang, false, span);
}

impl<'p> Ctx<'p> {
    /// `Box<str>::into_string()` and `String::into_boxed_str()` on a
    /// receiver of exactly that type: the type of the call, or None
    /// when the call is not one of these.
    pub(super) fn own_by_value_method(&mut self, e: ExprId, rt: Ty, name: Sym, args: &[ExprId], span: Span) -> Option<Ty> {
        let rt = self.deep(rt);
        let (b, ret) = match self.name(name) {
            "into_string" if own_parts(&self.tys, rt).is_some_and(|(k, x)| k == Owner::Box && x == STR) => (Builtin::OwnIntoString, STRING),
            "into_boxed_str" if rt == STRING => (Builtin::OwnFrom(Owner::Box), self.tys.mk_boxed(STR)),
            _ => return None,
        };
        self.set_res(e, Res::BuiltinMethod { b, derefs: 0, autoref: Autoref::None });
        self.check_args(&[], args, span);
        Some(ret)
    }
}

impl<'p> Ctx<'p> {
    /// Rejects an `Rc` or `Arc` fn on a pointer to `str` or `[T]` (the
    /// first param, behind a reference) that needs a sized value
    /// (`try_unwrap`, `into_inner`, `unwrap_or_clone`: rustc E0277), or
    /// that lust does not build for one yet (`downgrade`).
    /// True after reporting.
    pub(super) fn own_call_guard(&mut self, b: Builtin, params: &[Ty], span: Span) -> bool {
        use crate::check::sync::SyncOp as S;
        let sized_only = matches!(
            b,
            Builtin::RcTryUnwrap
                | Builtin::RcIntoInner
                | Builtin::RcUnwrapOrClone
                | Builtin::Sync(S::ArcTryUnwrap | S::ArcIntoInner | S::ArcUnwrapOrClone)
        );
        let not_yet = matches!(b, Builtin::RcDowngrade | Builtin::Sync(S::ArcDowngrade));
        if !sized_only && !not_yet {
            return false;
        }
        let Some(&pt) = params.first() else { return false };
        let t = self.deep(pt);
        let ptr = match self.kind(t) {
            TyKind::Ref(_, _, inner) => inner,
            _ => t,
        };
        // `Rc<dyn Tr>` keeps its older checks (check/mod.rs `call`).
        let Some(e) = shared_unsized_elem(&self.tys, ptr).or_else(|| match own_parts(&self.tys, ptr) {
            Some((Owner::Arc, e)) => Some(e),
            _ => None,
        }) else {
            return false;
        };
        // `Arc::downgrade` of an `Arc<dyn Tr>`, and `downgrade` of an Rc or
        // Arc of `str` or `[T]`, are built (cgen/own.rs).
        if b == Builtin::Sync(S::ArcDowngrade) && matches!(self.kind(e), TyKind::Dyn(..)) {
            return false;
        }
        if matches!(b, Builtin::RcDowngrade | Builtin::Sync(S::ArcDowngrade)) && matches!(self.kind(e), TyKind::Str | TyKind::Slice(_)) {
            return false;
        }
        let tn = self.tn(e);
        if sized_only {
            self.err(
                "L0110",
                span,
                format!("the size for values of type `{tn}` cannot be known, and this fn needs a sized value"),
                "Keep the shared pointer and read through it.",
            );
        } else {
            self.errs.push(crate::diag::not_yet("L0641", span, &format!("this fn on a shared pointer to `{tn}`")));
        }
        true
    }
}

/// Whether std's `Default` gives a value of `t` when it is an owning
/// pointer to `str` or `[T]` (always: the empty one; `Box<[T]>` is
/// generics.rs's), or to a dyn value (never). None for other types.
pub fn own_default(tys: &impl TyIntern, t: Ty) -> Option<bool> {
    let (k, e) = own_parts(tys, t)?;
    match tys.kind(e) {
        TyKind::Slice(_) if k == Owner::Box => None,
        TyKind::Str | TyKind::Slice(_) => Some(true),
        _ => Some(false),
    }
}

impl<'p> Ctx<'p> {
    /// True when `e` is `x.into()` or a `T::from(x)` call: its target
    /// comes from the expected type, so `Some(e)`, `Ok(e)`, and `Err(e)`
    /// pass their payload's expected type down (`Some("a".into())` as an
    /// `Option<Box<str>>`), as rustc's expected inputs do.
    pub(super) fn conversion_call(&self, e: ExprId) -> bool {
        match self.p.ast.expr(e) {
            Expr::MethodCall { name, args, generic_args, .. } => args.is_empty() && generic_args.is_empty() && self.name(*name) == "into",
            Expr::Call { callee, args } => {
                args.len() == 1
                    && matches!(self.p.ast.expr(*callee), Expr::Path(segs) if segs.len() >= 2 && segs.last().is_some_and(|s| self.name(s.name) == "from"))
            }
            _ => false,
        }
    }
}

impl<'p> Ctx<'p> {
    /// Binds the open element of a sequence source (`Vec<_>`, `[_; N]`,
    /// `&[_]`, `Box<[_]>`, with a variable anywhere in the element, such
    /// as `({integer}, String)`) to the element of the slice pointer
    /// `dst` (`Rc<[T]>`), the one std `From` impl that can apply. Rolls
    /// back when they do not unify.
    /// `Rc::<i32>::from(Box::new(5))`: alloc's `From<Box<T>>` fixes an
    /// open pointee of a `Box` source to the target's when `From<T>` (the
    /// `Rc<Box<_>>` reading) cannot apply, as rustc's one candidate does.
    fn unify_open_box(&mut self, dst: Ty, src: Ty) {
        let src = self.deep(src);
        let (Some(want), Some(have)) = (self.tys.rc_elem(dst), self.tys.boxed_elem(src)) else { return };
        if !self.tys.flags(have).contains_infer() {
            return;
        }
        let snap = self.snapshot_vars();
        let as_value = self.unify(want, src);
        self.rollback_vars(snap);
        if as_value || !self.unify(have, want) {
            self.rollback_vars(snap);
        }
    }

    fn unify_open_elem(&mut self, dst: Ty, src: Ty) {
        let Some(want) = shared_unsized_elem(&self.tys, dst).or_else(|| self.tys.boxed_elem(dst)) else { return };
        let TyKind::Slice(want) = self.kind(want) else { return };
        let src = self.deep(src);
        let inner = match self.kind(src) {
            TyKind::Ref(_, _, i) => i,
            _ => src,
        };
        let have = match self.kind(inner) {
            TyKind::Array(e, _) | TyKind::Slice(e) => Some(e),
            _ => self.tys.vec_elem(inner).or_else(|| self.tys.boxed_slice_elem(inner)),
        };
        let Some(have) = have.map(|h| self.deep(h)) else { return };
        if !self.tys.flags(have).contains_infer() {
            return;
        }
        let snap = self.snapshot_vars();
        if !self.unify(have, want) {
            self.rollback_vars(snap);
        }
    }
}

impl<'p> Ctx<'p> {
    /// Body end for `Rc::make_mut`, `Rc::get_mut`, and `Arc::make_mut`
    /// (`deferred_make_mut`): `make_mut` clones a shared value, so it
    /// needs std's `CloneToUninit` (`str` always, `[T]` for `T: Clone`,
    /// a sized `T` for `T: Clone`); a `dyn` pointee is not built yet.
    pub(super) fn finish_make_mut(&mut self) {
        for (t, span, clones) in std::mem::take(&mut self.deferred_make_mut) {
            let t = self.deep(t);
            let t = self.normalize_here(t);
            let need = match self.kind(t) {
                TyKind::Error | TyKind::Infer(_) | TyKind::Str => continue,
                TyKind::Dyn(..) => {
                    self.errs.push(generics::dyn_not_yet(span, "this `Rc` function on an `Rc<dyn Tr>`"));
                    continue;
                }
                TyKind::Slice(e) => self.deep(e),
                _ => t,
            };
            if clones && !has_clone(self.p, &self.tys, need) && !matches!(self.kind(need), TyKind::Error | TyKind::Infer(_)) {
                let tn = self.tn(need);
                self.err("L0112", span, format!("`{tn}` does not implement Clone"), "Add #[derive(Clone)] to the type.");
            }
        }
    }
}

/// The `dyn Tr` of a `RefCell<dyn Tr>` (an unsized cell: it lives behind
/// an `Rc`, `Arc`, `Box`, or reference).
pub fn dyn_cell_elem(tys: &impl TyIntern, t: Ty) -> Option<Ty> {
    let e = tys.refcell_elem(t)?;
    matches!(tys.kind(e), TyKind::Dyn(..)).then_some(e)
}

/// The `dyn Tr` of a `Mutex<dyn Tr>` (an unsized lock, behind a pointer).
pub fn dyn_lock_elem(tys: &impl TyIntern, t: Ty) -> Option<Ty> {
    match tys.lock_elem(t)? {
        (crate::ty::sync::SyncTy::Mutex, e) if matches!(tys.kind(e), TyKind::Dyn(..)) => Some(e),
        _ => None,
    }
}

/// The `dyn Tr` inside an unsized `RefCell<dyn Tr>` or `Mutex<dyn Tr>`.
pub fn dyn_wrapped_elem(tys: &impl TyIntern, t: Ty) -> Option<Ty> {
    dyn_cell_elem(tys, t).or_else(|| dyn_lock_elem(tys, t))
}

impl<'p> Ctx<'p> {
    /// The reject for a `RefCell<dyn Tr>` method other than `borrow` and
    /// `borrow_mut` (a `Mutex<dyn Tr>` one other than `lock` and
    /// `is_poisoned`): the by-value ones need a sized value (rustc
    /// E0277), the rest are not built for a dyn value yet. The call's type.
    pub(super) fn dyn_cell_method_reject(&mut self, name: &str, span: Span, args: &[ExprId]) -> Ty {
        for &a in args {
            self.expr(a, None);
        }
        if matches!(name, "into_inner" | "replace" | "swap" | "take" | "replace_with") {
            self.err(
                "L0110",
                span,
                format!("`{name}` needs a sized value, not a `dyn` one"),
                "Borrow or lock the cell and work through the guard.",
            );
        } else {
            self.errs.push(crate::diag::not_yet("L0641", span, &format!("`{name}` on a `RefCell<dyn Tr>` or `Mutex<dyn Tr>`")));
        }
        ERROR
    }
}
