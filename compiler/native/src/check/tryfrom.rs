// `TryFrom` and `TryInto` for the scalar types (fx-tryfrom), with
// core's exact pair set and error types (rustc 1.99, probed
// exhaustively over the scalar types).
//
// The scalar pairs are builtins like numeric `From`: selection
// answers them ahead of the impl index (`try_from_select`), the
// `Error` projection normalizes by the same table
// (`try_from_assoc`), `T::try_from(x)` lowers as a checked cast
// (`Res::TryFromCast`), and `x.try_into()` likewise
// (`PrimOpKind::TryIntoCast`). Everything else runs on the trait
// records: user impls, the blanket through `Into` (scalar widening
// is its `Infallible` case here; a user `From` fn wraps in `Ok`,
// `Res::TryFromOk`), and the blanket `TryInto` through `TryFrom`
// (method.rs bridges it to the `try_from` fn).
use super::*;

/// Core's error for one scalar `TryFrom` pair: the sort of the
/// `Error` type, or None when core has no impl for the pair.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TryErr {
    /// `core::convert::Infallible`: the pair converts by `From`.
    Infallible,
    /// `core::num::TryFromIntError`: a checked integer conversion,
    /// or an integer to `bool`.
    Int,
    /// `core::char::CharTryFromError`: `char` from `u32`.
    CharFrom,
    /// `core::char::TryFromCharError`: `u8`, `u16`, or `usize`
    /// from `char`.
    FromChar,
    /// `core::array::TryFromSliceError`: a slice to an array or
    /// array reference (tf-slice).
    Slice,
}

/// The error sort of core's `TryFrom<src> for dst` between scalar
/// kinds, or None when the pair has no impl. A pair `From` covers
/// (identity included) answers `Infallible` through the blanket;
/// every other integer pair is checked.
pub fn scalar_try_from(src: TyKind, dst: TyKind) -> Option<TryErr> {
    use IntTy::*;
    if Ctx::numeric_from_ok(src, dst) {
        return Some(TryErr::Infallible);
    }
    Some(match (src, dst) {
        (TyKind::Int(_), TyKind::Int(_)) => TryErr::Int,
        (TyKind::Int(Usize | Isize), TyKind::Bool) => return None,
        (TyKind::Int(_), TyKind::Bool) => TryErr::Int,
        (TyKind::Int(U32), TyKind::Char) => TryErr::CharFrom,
        (TyKind::Char, TyKind::Int(U8 | U16 | Usize)) => TryErr::FromChar,
        _ => return None,
    })
}

/// The shape of a slice-to-array pair (no `Copy` check, no
/// unification): the solver's `Error` normalization reads it on
/// types it cannot unify (traits/solve.rs).
pub fn slice_try_shape(tys: &impl crate::ty::TyIntern, src: Ty, dst: Ty) -> bool {
    let TyKind::Ref(_, sm, si) = tys.kind(src) else { return false };
    let TyKind::Slice(el) = tys.kind(si) else { return false };
    match tys.kind(dst) {
        TyKind::Array(el2, _) => el == el2,
        TyKind::Ref(_, dm, di) => (dm != Mut::Mut || sm == Mut::Mut) && matches!(tys.kind(di), TyKind::Array(el2, _) if el2 == el),
        _ => false,
    }
}

/// The shape of alloc's owned-to-array `TryFrom` pairs (no
/// unification), for the solver's `Error` normalization: the error
/// is the source type itself (`owned_try_parts`).
pub fn vec_try_shape(tys: &impl crate::ty::TyIntern, src: Ty, dst: Ty) -> bool {
    owned_try_parts(tys, src, dst).is_some_and(|(se, de)| se == de)
}

/// The element types of alloc's owned-to-array `TryFrom` pairs, by
/// shape: `TryFrom<Vec<T>> for [T; N]`, `TryFrom<Vec<T>> for Box<[T;
/// N]>`, and `TryFrom<Box<[T]>> for Box<[T; N]>` (no bound on `T`;
/// the source comes back in `Err` on a length miss). None for other
/// pairs.
pub fn owned_try_parts(tys: &impl crate::ty::TyIntern, src: Ty, dst: Ty) -> Option<(Ty, Ty)> {
    let arr_el = |t: Ty| match tys.kind(t) {
        TyKind::Array(el, _) => Some(el),
        _ => None,
    };
    if tys.builtin_of(src) == Some(BuiltinTy::Vec) {
        let se = tys.adt_first_ty_arg(src)?;
        let de = arr_el(dst).or_else(|| arr_el(tys.boxed_elem(dst)?))?;
        return Some((se, de));
    }
    let se = tys.boxed_slice_elem(src)?;
    Some((se, arr_el(tys.boxed_elem(dst)?)?))
}

/// True for the concrete scalar kinds the pair table covers.
fn scalar_kind(k: TyKind) -> bool {
    matches!(k, TyKind::Int(_) | TyKind::Float(_) | TyKind::Char | TyKind::Bool)
}

impl<'p> Ctx<'p> {
    /// The `Error` type of one scalar pair sort.
    pub(super) fn try_err_ty(&mut self, sort: TryErr) -> Ty {
        match sort {
            TryErr::Infallible => generics::infallible_ty(&mut self.tys),
            TryErr::Int => generics::try_from_int_error_ty(&mut self.tys),
            TryErr::CharFrom => generics::char_try_from_error_ty(&mut self.tys),
            TryErr::FromChar => generics::try_from_char_error_ty(&mut self.tys),
            TryErr::Slice => generics::try_from_slice_error_ty(&mut self.tys),
        }
    }

    /// Core's slice-to-array `TryFrom` pairs: `&[T]` and
    /// `&mut [T]` to `[T; N]` (`T: Copy`), `&[T]` to `&[T; N]`, and
    /// `&mut [T]` to `&mut [T; N]`. Some(Slice) for exactly those,
    /// with the elements unified; None otherwise.
    pub(super) fn slice_try_from(&mut self, src: Ty, dst: Ty) -> Option<TryErr> {
        let TyKind::Ref(_, sm, si) = self.kind(src) else { return None };
        let TyKind::Slice(el) = self.kind(si) else { return None };
        match self.kind(dst) {
            TyKind::Array(el2, _) => {
                let el = self.deep(el);
                if !self.unify(el, el2) || !crate::check::collect::is_copy(self.p, &self.tys, el) {
                    return None;
                }
            }
            TyKind::Ref(_, dm, di) => {
                let TyKind::Array(el2, _) = self.kind(di) else { return None };
                if dm == Mut::Mut && sm != Mut::Mut {
                    return None;
                }
                if !self.unify(el, el2) {
                    return None;
                }
            }
            _ => return None,
        }
        Some(TryErr::Slice)
    }

    /// Alloc's owned-to-array pairs (`owned_try_parts`: a Vec to an
    /// array or a boxed array, a boxed slice to a boxed array): the
    /// impl's `Error`, the source type itself, with the elements
    /// unified; None for any other pair.
    pub(super) fn vec_try_from(&mut self, src: Ty, dst: Ty) -> Option<Ty> {
        let (src, dst) = (self.deep(src), self.deep(dst));
        let (el, el2) = owned_try_parts(&self.tys, src, dst)?;
        if !self.unify(el, el2) {
            return None;
        }
        Some(src)
    }

    /// The `Error` type of core's array `TryFrom` pairs: a slice
    /// source (`slice_try_from`) or a Vec source (`vec_try_from`).
    pub(super) fn array_try_err(&mut self, src: Ty, dst: Ty) -> Option<Ty> {
        if let Some(sort) = self.slice_try_from(src, dst) {
            return Some(self.try_err_ty(sort));
        }
        self.vec_try_from(src, dst)
    }

    /// Selection for a `TryFrom` or `TryInto` obligation between
    /// scalar types, ahead of the solver (like numeric `From`):
    /// a checked cast with no impl record behind it. None when
    /// the pair is not two concrete scalars, so the solver runs
    /// (user impls, the blankets, the param env).
    pub(super) fn try_from_select(&mut self, tr: TraitRef, self_ty: Ty) -> Option<Selection> {
        let lang = &self.p.gx.lang;
        if tr.def != lang.try_from && tr.def != lang.try_into {
            return None;
        }
        let input = self.deep(self.tys.args(tr.args).get(1).and_then(|g| g.as_ty())?);
        let self_ty = self.deep(self_ty);
        let (src, dst) = if tr.def == lang.try_from { (input, self_ty) } else { (self_ty, input) };
        let (sk, dk) = (self.kind(src), self.kind(dst));
        // A parameter side is the env's or the solver's to prove
        // (`U: TryInto<u8>` in a template), never this table's.
        if matches!(sk, TyKind::Param(_) | TyKind::Projection(..)) || matches!(dk, TyKind::Param(_) | TyKind::Projection(..)) {
            return None;
        }
        // The slice-to-array pairs, ahead of the scalar table
        // (tf-slice): a checked length test with no impl record.
        if let Some(err_ty) = self.array_try_err(src, dst) {
            let assoc = if tr.def == lang.try_from { lang.assoc_try_from_error } else { lang.assoc_try_into_error };
            for (a, ga) in self.tys.bindings(tr.bindings).to_vec() {
                if a == assoc && ga.as_ty().is_some_and(|want| !self.unify(err_ty, want)) {
                    return Some(Selection::NoImpl);
                }
            }
            return Some(Selection::ParamEvidence(usize::MAX));
        }
        // A non-scalar side (`Even: TryFrom<{integer}>`) is the impl
        // index's: user impls and the blankets.
        let scalar_or_var = |k: TyKind| scalar_kind(k) || matches!(k, TyKind::Infer(_));
        if !scalar_or_var(sk) || !scalar_or_var(dk) {
            return None;
        }
        if matches!(sk, TyKind::Infer(_)) || matches!(dk, TyKind::Infer(_)) {
            // A literal or a target a later use fixes: the call
            // sites fall back or defer, and the queued obligation
            // re-asks once the variable binds.
            return Some(Selection::Ambiguous);
        }
        let sort = scalar_try_from(sk, dk)?;
        let err_ty = self.try_err_ty(sort);
        let assoc = if tr.def == lang.try_from { lang.assoc_try_from_error } else { lang.assoc_try_into_error };
        for (a, ga) in self.tys.bindings(tr.bindings).to_vec() {
            if a == assoc && ga.as_ty().is_some_and(|want| !self.unify(err_ty, want)) {
                return Some(Selection::NoImpl);
            }
        }
        Some(Selection::ParamEvidence(usize::MAX))
    }

    /// `<dst as TryFrom<src>>::Error` (or the `TryInto` spelling)
    /// for a scalar pair: the normalization the solver asks the
    /// context for (`SolveCx::builtin_assoc`).
    pub(super) fn try_from_assoc(&mut self, assoc: DefId, args: ArgsId) -> Option<Ty> {
        let lang = &self.p.gx.lang;
        let a: Vec<Ty> = self.tys.args(args).iter().filter_map(|g| g.as_ty()).collect();
        let [a0, a1] = a[..] else { return None };
        let (src, dst) = if assoc == lang.assoc_try_from_error {
            (a1, a0)
        } else if assoc == lang.assoc_try_into_error {
            (a0, a1)
        } else {
            return None;
        };
        if let Some(err) = self.array_try_err(src, dst) {
            return Some(err);
        }
        let (sk, dk) = (self.kind(src), self.kind(dst));
        let sort = scalar_try_from(sk, dk)?;
        Some(self.try_err_ty(sort))
    }

    /// A scalar or blanket `Dst::try_from(x)` (also `<Dst as
    /// TryFrom<Src>>::try_from(x)` through the hint, and
    /// `TryFrom::try_from(x)` with the target from the expected
    /// type): a checked cast, or an `Ok` around a `From` call.
    /// None when the call is another form, so the generic assoc
    /// search runs (user `TryFrom` impls).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn try_from_assoc_call(
        &mut self,
        e: ExprId,
        callee: ExprId,
        base: Ty,
        hint: Option<TraitRef>,
        name: Sym,
        args: &[ExprId],
        span: Span,
    ) -> Option<Ty> {
        let p = self.p;
        let lang = &p.gx.lang;
        if self.name(name) != "try_from" || args.len() != 1 || !hint.is_none_or(|h| h.def == lang.try_from) {
            return None;
        }
        // An instance replays the template's evidence like `from`.
        let hint = match self.replayed_trait_ref(e, lang.try_from) {
            Some(r) if hint.is_none() => Some(r),
            _ => hint,
        };
        let src = self.expr(args[0], None);
        let hint_input = hint.and_then(|h| self.tys.args(h.args).get(1).and_then(|g| g.as_ty())).map(|t| self.deep(t));
        let hint_ok = match hint_input {
            Some(hi) => self.unify(src, hi),
            None => true,
        };
        let src = self.deep(src);
        let (sk, dk) = (self.kind(src), self.kind(base));
        // The slice-to-array forms (tf-slice): the same checked
        // cast res, lowered by the destination type.
        if hint_ok && let Some(err) = self.array_try_err(src, base) {
            self.set_res(callee, Res::TryFromCast);
            self.set_res(e, Res::TryFromCast);
            return Some(self.tys.mk_result(base, err));
        }
        if hint_ok && scalar_kind(dk) {
            // An unsuffixed literal argument takes its fallback
            // type when that pair converts, as in rustc.
            let (sk, src_ok) = if sk.is_int_infer() || sk.is_float_infer() {
                let fb = if sk.is_int_infer() { int(IntTy::I32) } else { self.tys.float_ty(FloatTy::F64) };
                let fk = self.kind(fb);
                if scalar_try_from(fk, dk).is_some() && self.unify(src, fb) { (fk, true) } else { (sk, false) }
            } else {
                (sk, scalar_kind(sk))
            };
            if src_ok {
                return match scalar_try_from(sk, dk) {
                    Some(sort) => {
                        self.set_res(callee, Res::TryFromCast);
                        self.set_res(e, Res::TryFromCast);
                        let err = self.try_err_ty(sort);
                        Some(self.tys.mk_result(base, err))
                    }
                    None => {
                        let (sn, dn) = (self.tn(src), self.tn(base));
                        self.err(
                            "L0612",
                            span,
                            format!("no impl of `TryFrom<{sn}>` for `{dn}`"),
                            "Change the source or target type.",
                        );
                        Some(ERROR)
                    }
                };
            }
            if (sk.is_int_infer() || sk.is_float_infer()) && scalar_kind(dk) {
                // No scalar impl even at the fallback type.
                let fb = if sk.is_int_infer() { int(IntTy::I32) } else { self.tys.float_ty(FloatTy::F64) };
                let (sn, dn) = (self.tn(fb), self.tn(base));
                self.err(
                    "L0612",
                    span,
                    format!("no impl of `TryFrom<{sn}>` for `{dn}`"),
                    "Add a literal suffix, or change the target type.",
                );
                return Some(ERROR);
            }
            return None;
        }
        // The blanket through `From`: identity moves, a user or
        // template-bound `From` fn wraps in `Ok`. The scalar pairs
        // are the checked cast above, so only non-scalar inputs or
        // targets reach here.
        if src == base && matches!(sk, TyKind::Adt(..) | TyKind::Tuple(_) | TyKind::Array(..)) {
            self.set_res(callee, Res::TryFromOk(None));
            self.set_res(e, Res::TryFromOk(None));
            let err = self.try_err_ty(TryErr::Infallible);
            return Some(self.tys.mk_result(base, err));
        }
        if matches!(sk, TyKind::Infer(_) | TyKind::Error) || matches!(dk, TyKind::Infer(_) | TyKind::Error) {
            return None;
        }
        let args_id = self.tys.intern_args(&[GenericArg::Ty(base), GenericArg::Ty(src)]);
        let tr = TraitRef { def: lang.from, args: args_id, bindings: EMPTY_BINDINGS };
        let snap = self.snapshot_vars();
        let obl_len = self.obl_len();
        let sites_len = self.generic_sites.len();
        let mut env_pick: Option<usize> = None;
        let found = match self.select_obligation_at(tr, span, 0) {
            Selection::Impl { id, args: impl_args, .. } => {
                // Only a real impl fn wraps: a synthetic lang impl
                // (identity, a builtin `From`) has none this call
                // can run, except the identity handled above.
                if p.gx.index.get(id).synthetic == crate::traits::index::Synthetic::Lang {
                    None
                } else {
                    self.method_instance_for_impl(e, id, tr, impl_args, "from", span)
                }
            }
            Selection::ParamEvidence(i) if i != usize::MAX => {
                env_pick = Some(i);
                self.stub_instance(e, tr, p.interner.intern("from"), span)
            }
            _ => None,
        };
        match found {
            Some((local, params, _ret)) => {
                if env_pick.is_some() {
                    self.record_pick_env(e, lang.from, 0, ReceiverForm::Value, env_pick);
                }
                self.set_res(callee, Res::TryFromOk(Some(local)));
                self.set_res(e, Res::TryFromOk(Some(local)));
                self.check_args(&params, args, span);
                let err = self.try_err_ty(TryErr::Infallible);
                Some(self.tys.mk_result(base, err))
            }
            None => {
                self.rollback_vars(snap);
                self.obl_truncate(obl_len);
                self.generic_sites.truncate(sites_len);
                None
            }
        }
    }

    /// A scalar `<Src as TryInto<Dst>>::try_into(x)`: the same
    /// checked cast as `Dst::try_from(x)`, in associated form.
    /// None when the written trait names no scalar target, so the
    /// generic assoc search runs.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn try_into_assoc_call(
        &mut self,
        e: ExprId,
        callee: ExprId,
        base: Ty,
        hint: Option<TraitRef>,
        name: Sym,
        args: &[ExprId],
        span: Span,
    ) -> Option<Ty> {
        let lang = &self.p.gx.lang;
        if self.name(name) != "try_into" || args.len() != 1 {
            return None;
        }
        let hint = hint.filter(|h| h.def == lang.try_into)?;
        let dst = self.deep(self.tys.args(hint.args).get(1).and_then(|g| g.as_ty())?);
        let (sk, dk) = (self.kind(base), self.kind(dst));
        if let Some(err) = self.array_try_err(base, dst) {
            let src = self.expr(args[0], Some(base));
            if !self.unify(src, base) {
                self.mismatch(span, base, src);
                return Some(ERROR);
            }
            self.set_res(callee, Res::TryFromCast);
            self.set_res(e, Res::TryFromCast);
            return Some(self.tys.mk_result(dst, err));
        }
        if !scalar_kind(sk) || !scalar_kind(dk) {
            return None;
        }
        let src = self.expr(args[0], Some(base));
        if !self.unify(src, base) {
            self.mismatch(span, base, src);
            return Some(ERROR);
        }
        match scalar_try_from(sk, dk) {
            Some(sort) => {
                self.set_res(callee, Res::TryFromCast);
                self.set_res(e, Res::TryFromCast);
                let err = self.try_err_ty(sort);
                Some(self.tys.mk_result(dst, err))
            }
            None => {
                let (sn, dn) = (self.tn(base), self.tn(dst));
                self.err("L0612", span, format!("no impl of `TryFrom<{sn}>` for `{dn}`"), "Change the source or target type.");
                Some(ERROR)
            }
        }
    }

    /// `x.try_into()` to a scalar target: the checked cast. The
    /// target comes from the template's evidence, the expected
    /// `Result` type, or a later use (a deferred call the body
    /// end settles, like the generic search's). None when the
    /// receiver or a known target is not scalar, so the trait
    /// search runs (user `TryFrom` impls through the blanket).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn prim_try_into_method(
        &mut self,
        e: ExprId,
        base: Ty,
        derefs: u8,
        name: Sym,
        name_span: Span,
        args: &[ExprId],
        turbofish: &[ast::GenericArg],
        expected: Option<Ty>,
        builtin_only: bool,
    ) -> Option<Ty> {
        let p = self.p;
        let lang = &p.gx.lang;
        if !turbofish.is_empty() || self.name(name) != "try_into" || derefs > 0 {
            return None;
        }
        let src_kind = self.kind(base);
        if !scalar_kind(src_kind) {
            return None;
        }
        let replayed = self
            .replayed_trait_ref(e, lang.try_into)
            .and_then(|r| self.tys.args(r.args).get(1).and_then(|g| g.as_ty()))
            .map(|t| self.deep(t));
        let exp_pair = expected.map(|t| self.deep(t)).and_then(|t| self.tys.result_args(t));
        let known = replayed.filter(|&t| scalar_kind(self.kind(t))).or_else(|| exp_pair.map(|(ok, _)| self.deep(ok)));
        if let Some(dst) = known {
            let dk = self.kind(dst);
            if !scalar_kind(dk) {
                return None;
            }
            let Some(sort) = scalar_try_from(src_kind, dk) else {
                let (sn, dn) = (self.tn(base), self.tn(dst));
                self.err("L0612", name_span, format!("no impl of `TryFrom<{sn}>` for `{dn}`"), "Change the target type.");
                return Some(ERROR);
            };
            if !builtin_only && self.prim_lang_guard(name, name_span, args, lang.try_into, base) {
                return Some(ERROR);
            }
            self.check_args(&[], args, name_span);
            self.set_res(e, Res::PrimOp { kind: PrimOpKind::TryIntoCast, derefs });
            let err = self.try_err_ty(sort);
            return Some(self.tys.mk_result(dst, err));
        }
        // The target is a later use's to fix (`x.try_into()
        // .unwrap_or(9u8)`): type the call with fresh variables,
        // queue the obligation, and let the body end settle the
        // deferred call (a scalar target keeps this cast, a user
        // `TryFrom` target re-points it at the impl fn).
        if !args.is_empty() {
            self.check_args(&[], args, name_span);
        }
        let dst = self.new_var(false);
        // The error type stays the trait's projection, not a
        // variable a `?` could bind to the return error type: it
        // normalizes once the target is known (the `?` conversion
        // deferral in `try_expr` reads it then).
        let proj_args = self.tys.intern_args(&[GenericArg::Ty(base), GenericArg::Ty(dst)]);
        let err = self.tys.intern(TyKind::Projection(AssocId(lang.assoc_try_into_error), proj_args));
        let args_id = self.tys.intern_args(&[GenericArg::Ty(base), GenericArg::Ty(dst)]);
        let tr = TraitRef { def: lang.try_into, args: args_id, bindings: EMPTY_BINDINGS };
        self.obl.push_trait_at(tr, name_span, 0);
        method::defer_trait_method(self, e, tr, name, Vec::new(), derefs, Autoref::None);
        self.set_res(e, Res::PrimOp { kind: PrimOpKind::TryIntoCast, derefs });
        Some(self.tys.mk_result(dst, err))
    }

    /// `slice.try_into()` to an array or array-reference target
    /// (tf-slice): the same checked cast res as the scalar form,
    /// lowered by the destination type. The receiver is a slice
    /// place, a slice reference, or an array (through its slice);
    /// a `&mut [T; N]` target needs a written `&mut [T]` receiver
    /// (a mutable place form is not taken yet). None when the
    /// receiver is none of these, so the trait search runs.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn slice_try_into_method(
        &mut self,
        e: ExprId,
        rt: Ty,
        base: Ty,
        derefs: u8,
        name: Sym,
        name_span: Span,
        args: &[ExprId],
        turbofish: &[ast::GenericArg],
        expected: Option<Ty>,
        builtin_only: bool,
    ) -> Option<Ty> {
        let p = self.p;
        let lang = &p.gx.lang;
        // A turbofish on `try_into` is rustc E0107 (the target is
        // the trait's parameter): the normal path reports it.
        if self.name(name) != "try_into" || !turbofish.is_empty() {
            return None;
        }
        // The src type the `TryFrom` pair is read on: a written
        // slice reference as itself; a slice place as a shared
        // slice of itself. An array receiver has no slice pair
        // (rustc finds no `TryFrom<[T; N]>` there).
        let src = match self.kind(rt) {
            TyKind::Ref(_, _, i) if matches!(self.kind(i), TyKind::Slice(_)) => rt,
            _ => match self.kind(base) {
                TyKind::Slice(el) => {
                    let sl = self.tys.intern(TyKind::Slice(el));
                    self.tys.mk_ref(false, sl)
                }
                _ => return None,
            },
        };
        let replayed = self
            .replayed_trait_ref(e, lang.try_into)
            .and_then(|r| self.tys.args(r.args).get(1).and_then(|g| g.as_ty()))
            .map(|t| self.deep(t));
        let exp_pair = expected.map(|t| self.deep(t)).and_then(|t| self.tys.result_args(t));
        let known = replayed.or_else(|| exp_pair.map(|(ok, _)| self.deep(ok)));
        if let Some(dst) = known {
            // A target that is no array form is another conversion's.
            let arrayish = match self.kind(dst) {
                TyKind::Array(..) => true,
                TyKind::Ref(_, _, i) => matches!(self.kind(i), TyKind::Array(..)),
                _ => false,
            };
            if !arrayish {
                return None;
            }
            let Some(sort) = self.slice_try_from(src, dst) else {
                let (sn, dn) = (self.tn(src), self.tn(dst));
                self.err("L0612", name_span, format!("no impl of `TryFrom<{sn}>` for `{dn}`"), "Change the target type.");
                return Some(ERROR);
            };
            if !builtin_only && self.prim_lang_guard(name, name_span, args, lang.try_into, base) {
                return Some(ERROR);
            }
            self.check_args(&[], args, name_span);
            self.set_res(e, Res::PrimOp { kind: PrimOpKind::TryIntoCast, derefs });
            let err = self.try_err_ty(sort);
            return Some(self.tys.mk_result(dst, err));
        }
        // The target is a later use's to fix: as the scalar form,
        // with the slice as the obligation's Self (the pair the
        // selection answers) and the error as its projection.
        if !args.is_empty() {
            self.check_args(&[], args, name_span);
        }
        let dst = self.new_var(false);
        let proj_args = self.tys.intern_args(&[GenericArg::Ty(src), GenericArg::Ty(dst)]);
        let err = self.tys.intern(TyKind::Projection(AssocId(lang.assoc_try_into_error), proj_args));
        let args_id = self.tys.intern_args(&[GenericArg::Ty(src), GenericArg::Ty(dst)]);
        // The obligation carries the slice error binding: a
        // target that settles through another impl (a Vec through
        // the blanket) then fails selection cleanly instead of
        // lowering this cast against a stale result type.
        let slice_err = self.try_err_ty(TryErr::Slice);
        let bindings = self.tys.intern_bindings(&[(lang.assoc_try_into_error, GenericArg::Ty(slice_err))]);
        let tr = TraitRef { def: lang.try_into, args: args_id, bindings };
        self.obl.push_trait_at(tr, name_span, 0);
        method::defer_trait_method(self, e, tr, name, Vec::new(), derefs, Autoref::None);
        self.set_res(e, Res::PrimOp { kind: PrimOpKind::TryIntoCast, derefs });
        Some(self.tys.mk_result(dst, err))
    }

    /// `vec.try_into()` to an array target: core's `TryFrom<Vec<T>>
    /// for [T; N]` (and the boxed-array pairs from a Vec or a `Box<[T]>`,
    /// `owned_try_parts`), the checked cast res lowered by the
    /// destination type, with the source back in `Err` on a length
    /// miss. Only a value receiver (rustc picks the blanket `TryInto` at the
    /// first autoderef step, so `(&v).try_into()` is no Vec pair).
    /// None for any other receiver or a known non-array target, so
    /// the trait search runs (user `TryFrom<Vec<..>>` impls).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn vec_try_into_method(
        &mut self,
        e: ExprId,
        base: Ty,
        derefs: u8,
        name: Sym,
        name_span: Span,
        args: &[ExprId],
        turbofish: &[ast::GenericArg],
        expected: Option<Ty>,
        builtin_only: bool,
    ) -> Option<Ty> {
        let p = self.p;
        let lang = &p.gx.lang;
        let owned_src = self.tys.builtin_of(base) == Some(BuiltinTy::Vec) || self.tys.boxed_slice_elem(base).is_some();
        if self.name(name) != "try_into" || !turbofish.is_empty() || derefs > 0 || !owned_src {
            return None;
        }
        let replayed = self
            .replayed_trait_ref(e, lang.try_into)
            .and_then(|r| self.tys.args(r.args).get(1).and_then(|g| g.as_ty()))
            .map(|t| self.deep(t));
        let exp_pair = expected.map(|t| self.deep(t)).and_then(|t| self.tys.result_args(t));
        let known = replayed.or_else(|| exp_pair.map(|(ok, _)| self.deep(ok)));
        if let Some(dst) = known {
            let boxed_arr = self.tys.boxed_elem(dst).is_some_and(|b| matches!(self.kind(b), TyKind::Array(..)));
            if !matches!(self.kind(dst), TyKind::Array(..)) && !boxed_arr {
                return None;
            }
            let Some(err) = self.vec_try_from(base, dst) else {
                let (sn, dn) = (self.tn(base), self.tn(dst));
                self.err("L0612", name_span, format!("no impl of `TryFrom<{sn}>` for `{dn}`"), "Change the target type.");
                return Some(ERROR);
            };
            if !builtin_only && self.prim_lang_guard(name, name_span, args, lang.try_into, base) {
                return Some(ERROR);
            }
            self.check_args(&[], args, name_span);
            self.set_res(e, Res::PrimOp { kind: PrimOpKind::TryIntoCast, derefs });
            return Some(self.tys.mk_result(dst, err));
        }
        // The target is a later use's to fix, as the scalar form: the
        // error stays the trait's projection, and the body end either
        // keeps this cast (an array target) or re-points the call at
        // a user `TryFrom<Vec<..>>` impl fn.
        if !args.is_empty() {
            self.check_args(&[], args, name_span);
        }
        let dst = self.new_var(false);
        let proj_args = self.tys.intern_args(&[GenericArg::Ty(base), GenericArg::Ty(dst)]);
        let err = self.tys.intern(TyKind::Projection(AssocId(lang.assoc_try_into_error), proj_args));
        let args_id = self.tys.intern_args(&[GenericArg::Ty(base), GenericArg::Ty(dst)]);
        let tr = TraitRef { def: lang.try_into, args: args_id, bindings: EMPTY_BINDINGS };
        self.obl.push_trait_at(tr, name_span, 0);
        method::defer_trait_method(self, e, tr, name, Vec::new(), derefs, Autoref::None);
        self.set_res(e, Res::PrimOp { kind: PrimOpKind::TryIntoCast, derefs });
        Some(self.tys.mk_result(dst, err))
    }
}
