// Conversion trait methods on builtin values (W-STD-B): `as_ref`,
// `as_mut` (AsRef, AsMut), `borrow`, `borrow_mut` (Borrow,
// BorrowMut), and `to_owned` (ToOwned), with the impl sets core and
// alloc declare for the types lust has (generics.rs lang facts).
//
// Self is bound as rustc's probe binds it: the receiver with one
// reference peeled (the by-value step of a `&self` method), else the
// receiver by autoref. AsRef and AsMut forward through references;
// Borrow does not (`impl Borrow<T> for &T` is its own impl). The trait
// argument comes from the template's bound in an instance of a generic
// body, else from the expected type, else from a single impl; several
// impls and no expected type are rustc's E0283 (L0610 here).
//
// Also `x.into()` and the std `From` impls between builtin types
// (`std_from`): selection sees their records (generics.rs), calls are
// `Builtin::ConvInto`, which lower/iter.rs expands per conversion.
use super::*;

/// One of std's `From` impls between builtin types (W-STD-B), as
/// `dst: From<src>`: how the conversion runs (lower/iter.rs).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FromConv {
    /// `impl<T> From<T> for T`: the value itself.
    Same,
    /// String from `&str` or `&mut str`.
    StringFromStr,
    /// String from `&String`.
    StringFromRefString,
    /// String from a char.
    StringFromChar,
    /// String from `Cow<str>`, Vec<T> from `Cow<[T]>`: `into_owned`.
    CowIntoOwned,
    /// Vec<u8> from a String (its buffer).
    BytesFromString,
    /// Vec<u8> from `&str`.
    BytesFromStr,
    /// Vec<T> from `&[T]` or `&mut [T]` (T: Clone).
    VecFromSlice,
    /// Vec<T> from `[T; N]`, `&[T; N]`, or `&mut [T; N]` (by
    /// reference, T: Clone).
    VecFromArray,
    /// Vec<T> from `Box<[T]>` (its buffer).
    VecFromBoxSlice,
    /// `Box<[T]>` from Vec<T>.
    BoxSliceFromVec,
    /// `Option<T>` from T.
    Some,
    /// `Box<T>` from T.
    BoxNew,
    /// `Rc<T>` from T.
    RcNew,
    /// `Cow<str>` from `&str`, `Cow<[T]>` from `&[T]`: `Borrowed`.
    CowBorrowed,
    /// `Cow<str>` from `&String`, `Cow<[T]>` from `&Vec<T>`: `Borrowed`
    /// of the view.
    CowBorrowedView,
    /// `Cow<str>` from String, `Cow<[T]>` from Vec<T>: `Owned`.
    CowOwned,
    /// `Box`, `Rc`, or `Arc` of `str` or `[T]` (w-unsized, check/own.rs).
    Own(crate::check::own::Owner),
    /// String from `Box<str>`: its buffer.
    StringFromBoxStr,
    /// An integer from its `NonZero` (core's `From<NonZero<T>> for T`).
    NonZeroGet,
    /// `Rc<T>` or `Arc<T>` from `Box<T>` for a sized or `dyn` `T`
    /// (alloc's `impl<T: ?Sized> From<Box<T>>`; `str` and `[T]` are
    /// `Own`): the value moves into a new counted allocation, the
    /// vtable stays, and the box is freed without a drop.
    OwnFromBox(crate::check::own::Owner),
}

/// The std `From` impl `dst: From<src>` between builtin types, if std
/// has one (both types fully known). Element bounds are std's: the
/// cloning forms need `T: Clone`. Numeric widening is `numeric_from_ok`
/// (no runtime conversion beyond a cast), user impls are records.
pub fn std_from(p: &Program, tys: &impl TyIntern, dst: Ty, src: Ty) -> Option<FromConv> {
    use FromConv as F;
    if dst == src {
        return Some(F::Same);
    }
    let str_like = |t: Ty| matches!(tys.kind(t), TyKind::Ref(_, _, s) if tys.kind(s) == TyKind::Str);
    let ref_of = |t: Ty| match tys.kind(t) {
        TyKind::Ref(_, m, inner) => Some((m, inner)),
        _ => None,
    };
    let slice_elem = |t: Ty| match tys.kind(t) {
        TyKind::Slice(e) => Some(e),
        _ => None,
    };
    let cow = crate::check::collect::cow_arg(tys, src);
    if tys.is_string_ty(dst) {
        return match () {
            _ if str_like(src) => Some(F::StringFromStr),
            _ if ref_of(src).is_some_and(|(m, i)| !m.is_mut() && tys.is_string_ty(i)) => Some(F::StringFromRefString),
            _ if src == CHAR => Some(F::StringFromChar),
            _ if cow.is_some_and(|b| tys.kind(b) == TyKind::Str) => Some(F::CowIntoOwned),
            _ if crate::check::own::own_parts(tys, src).is_some_and(|(k, e)| k == crate::check::own::Owner::Box && tys.kind(e) == TyKind::Str) => {
                Some(F::StringFromBoxStr)
            }
            _ => None,
        };
    }
    if let Some(el) = tys.vec_elem(dst) {
        if el == int(IntTy::U8) && tys.is_string_ty(src) {
            return Some(F::BytesFromString);
        }
        if el == int(IntTy::U8) && str_like(src) {
            return Some(F::BytesFromStr);
        }
        if let Some((_, inner)) = ref_of(src)
            && slice_elem(inner) == Some(el)
        {
            return has_clone(p, tys, el).then_some(F::VecFromSlice);
        }
        if let TyKind::Array(e2, _) = tys.kind(src)
            && e2 == el
        {
            return Some(F::VecFromArray);
        }
        if let Some((_, inner)) = ref_of(src)
            && matches!(tys.kind(inner), TyKind::Array(e2, _) if e2 == el)
        {
            return has_clone(p, tys, el).then_some(F::VecFromArray);
        }
        if tys.boxed_slice_elem(src) == Some(el) {
            return Some(F::VecFromBoxSlice);
        }
        if cow.and_then(slice_elem) == Some(el) {
            return has_clone(p, tys, el).then_some(F::CowIntoOwned);
        }
        return None;
    }
    if let Some(el) = tys.boxed_slice_elem(dst)
        && tys.vec_elem(src) == Some(el)
    {
        return Some(F::BoxSliceFromVec);
    }
    if let Some(k) = crate::check::own::own_from(p, tys, dst, src) {
        return Some(F::Own(k));
    }
    if let Some(k) = crate::check::own::own_from_box(tys, dst, src) {
        return Some(F::OwnFromBox(k));
    }
    if tys.option_elem(dst) == Some(src) {
        return Some(F::Some);
    }
    if matches!(tys.kind(dst), TyKind::Int(_)) && crate::check::generics::nonzero_elem(tys, src) == Some(dst) {
        return Some(F::NonZeroGet);
    }
    if tys.boxed_elem(dst) == Some(src) {
        return Some(F::BoxNew);
    }
    if tys.rc_elem(dst) == Some(src) {
        return Some(F::RcNew);
    }
    if let Some(b) = crate::check::collect::cow_arg(tys, dst) {
        // The two generic impls of alloc::borrow, for every
        // `B: ?Sized + ToOwned`: `From<&B>` is `Borrowed`, and
        // `From<<B as ToOwned>::Owned>` is `Owned`. The Owned side
        // answers here for the shapes structure decides (str, slices,
        // a sized Clone `B` whose Owned is itself, and a mono
        // instance whose Owned field is already concrete); a written
        // `ToOwned` of any other shape is the checker's normalized
        // answer in `std_from_here`.
        if ref_of(src).is_some_and(|(m, i)| !m.is_mut() && i == b) {
            if let Some(el) = slice_elem(b)
                && !has_clone(p, tys, el)
            {
                return None;
            }
            return Some(F::CowBorrowed);
        }
        if src == b && crate::check::collect::is_sized_ty(tys, b) && has_clone(p, tys, b) {
            return Some(F::CowOwned);
        }
        if let Some(a) = tys.local_adt(dst)
            && let Some(f) = p.adt(a).variants.get(1).and_then(|v| v.fields.first())
            && f.ty == src
            && !matches!(tys.kind(src), TyKind::Projection(..))
        {
            return Some(F::CowOwned);
        }
        if tys.kind(b) == TyKind::Str {
            return match () {
                _ if tys.is_string_ty(src) => Some(F::CowOwned),
                _ if ref_of(src).is_some_and(|(m, i)| !m.is_mut() && tys.is_string_ty(i)) => Some(F::CowBorrowedView),
                _ => None,
            };
        }
        let el = slice_elem(b)?;
        if !has_clone(p, tys, el) {
            return None;
        }
        return match () {
            _ if tys.vec_elem(src) == Some(el) => Some(F::CowOwned),
            _ if ref_of(src).is_some_and(|(m, i)| !m.is_mut() && tys.vec_elem(i) == Some(el)) => Some(F::CowBorrowedView),
            _ => None,
        };
    }
    None
}

/// One conversion trait method.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Conv {
    AsRef,
    AsMut,
    Borrow,
    BorrowMut,
}

impl<'p> Ctx<'p> {
    /// `as_ref`, `as_mut`, `borrow`, `borrow_mut`, and `to_owned` on a
    /// builtin receiver (user Adts and parameters resolve their impls
    /// in the generic method search first). None when the call is not
    /// one of these, so the other dispatch runs (RefCell's inherent
    /// `borrow` when Borrow is not in scope).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn convert_method(
        &mut self,
        e: ExprId,
        recv: ExprId,
        rt: Ty,
        name: Sym,
        name_span: Span,
        args: &[ExprId],
        expected: Option<Ty>,
    ) -> Option<Ty> {
        let p = self.p;
        let lang = p.gx.lang;
        let n = p.interner.get(name);
        // `clone_into` is the one of these with an argument.
        if n != "clone_into" && !args.is_empty() {
            return None;
        }
        let rt = self.deep(rt);
        if matches!(n, "deref" | "deref_mut") {
            return self.deref_call(e, recv, rt, n == "deref_mut", name_span);
        }
        // Self: where the template's pick put it (an instance of a
        // generic body), else rustc's probe order (`conv_self_step`).
        let conv_traits = [lang.as_ref, lang.as_mut, lang.borrow, lang.borrow_mut, lang.to_owned];
        let replayed = self.replay_pick(e).filter(|pk| conv_traits.contains(&pk.trait_def));
        let (self_ty, self_derefs) = match replayed {
            Some(pick) => {
                let mut t = rt;
                for _ in 0..pick.derefs {
                    t = match self.tys.kind(t) {
                        TyKind::Ref(_, _, inner) => self.deep(inner),
                        _ => return None,
                    };
                }
                (t, pick.derefs)
            }
            None => self.conv_self_step(e, rt, n)?,
        };
        // A user Adt's own impls answer in the generic method search
        // first. Only core's reflexive `impl<T> Borrow<T> for T` (and
        // BorrowMut) is left for it here: a synthetic impl has no
        // method fn to call, so the search counts it as a miss
        // (`p.borrow()`, or `k.borrow()` in `K: Borrow<Q>` at K = P).
        // Next to a user `impl Borrow<str> for P` a plain `p.borrow()`
        // is ambiguous in rustc (E0283), so it stays a miss here; an
        // instance replays its template's pick, which fixed the input.
        let reflexive = match n {
            "borrow" | "borrow_mut" => {
                let tr = if n == "borrow" { lang.borrow } else { lang.borrow_mut };
                replayed.is_some() || !self.tys.adt_parts(self_ty).is_some_and(|(def, _)| user_impl_for_adt(p, tr, def))
            }
            _ => false,
        };
        // A Clone Adt's `to_owned` is core's blanket `impl<T: Clone>
        // ToOwned for T` (a user impl would overlap it).
        let blanket_owned = n == "to_owned" && has_clone(p, &self.tys, self_ty);
        if (self.tys.local_adt(self_ty).is_some() && !reflexive && !blanket_owned)
            || matches!(self.tys.kind(self_ty), TyKind::Param(_) | TyKind::Projection(..))
        {
            return None;
        }
        if n == "clone_into" {
            if args.len() != 1 {
                return None;
            }
            return self.clone_into_call(e, recv, self_ty, self_derefs, name_span, args);
        }
        // A builtin with an inherent `borrow` of its own (RefCell and its
        // kin) wins at its step: rustc probes inherent methods first.
        // Only the types below have none.
        let plain = |cx: &Self, t: Ty| {
            matches!(
                cx.tys.builtin_of(t),
                None | Some(BuiltinTy::String | BuiltinTy::Vec | BuiltinTy::Option | BuiltinTy::Result | BuiltinTy::Boxed | BuiltinTy::Rc)
                    | Some(BuiltinTy::Sync(crate::ty::sync::SyncTy::Arc))
            )
        };
        let conv = match n {
            "as_ref" => Conv::AsRef,
            "as_mut" => Conv::AsMut,
            // An instance of a `K: Borrow<Q>` body calling `k.borrow()`
            // on a builtin key lands here with no `use` in the module:
            // the template's bound speaks for scope (method.rs
            // `in_template_env`).
            "borrow"
                if (super::method::trait_in_scope_pub(self, lang.borrow) || self.enclosing_impl_trait_pub() == Some(lang.borrow))
                    && plain(self, self_ty) =>
            {
                Conv::Borrow
            }
            "borrow_mut"
                if (super::method::trait_in_scope_pub(self, lang.borrow_mut) || self.enclosing_impl_trait_pub() == Some(lang.borrow_mut))
                    && plain(self, self_ty) =>
            {
                Conv::BorrowMut
            }
            "to_owned" => return self.to_owned_call(e, recv, rt, self_ty, self_derefs, name_span),
            _ => return None,
        };
        // AsRef and AsMut forward `&T` and `&mut T` to T's impls.
        let (core, core_derefs) = if matches!(conv, Conv::AsRef | Conv::AsMut) {
            let mut c = self_ty;
            let mut d = self_derefs;
            while let TyKind::Ref(_, _, inner) = self.tys.kind(c) {
                c = self.deep(inner);
                d += 1;
            }
            (c, d)
        } else {
            (self_ty, self_derefs)
        };
        let cands = self.conv_targets(conv, core);
        if cands.is_empty() {
            return None;
        }
        let trait_def = match conv {
            Conv::AsRef => lang.as_ref,
            Conv::AsMut => lang.as_mut,
            Conv::Borrow => lang.borrow,
            Conv::BorrowMut => lang.borrow_mut,
        };
        let mutable = matches!(conv, Conv::AsMut | Conv::BorrowMut);
        // The trait argument: the template's bound, the one impl (rustc
        // selects it before the result coerces: `let s: &str =
        // boxed_string.as_ref()` is `&String`, deref-coerced), or the
        // expected type among several impls.
        // The template's own bound first (its env predicate, by index):
        // two bounds can name the same Self in an instance (`P:
        // AsRef<Path>, C: AsRef<[u8]>` at `P = C = &str`; W-SHIM).
        let replayed = self.replayed_trait_ref(e, trait_def).and_then(|tr| self.tys.args(tr.args).get(1).and_then(|g| g.as_ty()));
        let bound = replayed.or_else(|| self.instance_bound_arg(trait_def, self_ty));
        let want = bound.or_else(|| {
            if cands.len() == 1 {
                return Some(cands[0]);
            }
            let ex = self.deep(expected?);
            match self.tys.kind(ex) {
                TyKind::Ref(_, m, u) if m.is_mut() == mutable || !mutable => Some(self.deep(u)),
                _ => None,
            }
        });
        let target = match want {
            Some(u) => match cands.iter().copied().find(|&c| c == u).or_else(|| cands.iter().copied().find(|&c| self.unify(c, u))) {
                Some(c) => c,
                None => {
                    let (tn, un) = (self.tn(self_ty), self.tn(u));
                    let tr = self.trait_name(trait_def);
                    self.err("L0612", name_span, format!("no impl of `{tr}<{un}>` for `{tn}`"), "Change the target type.");
                    return Some(ERROR);
                }
            },
            None if cands.len() == 1 => cands[0],
            None => {
                self.err(
                    "L0610",
                    name_span,
                    format!("type annotations needed: `{n}` on `{}` has several impls", self.tn(self_ty)),
                    "Annotate the result type, for example `let s: &str = x.as_ref();`.",
                );
                return Some(ERROR);
            }
        };
        let b = self.conv_builtin(core, target);
        let autoref = if mutable { Autoref::RefMut } else { Autoref::Ref };
        self.check_packed_borrow(recv, name_span);
        self.set_res(e, Res::BuiltinMethod { b, derefs: core_derefs, autoref });
        Some(self.tys.mk_ref(mutable, target))
    }

    /// The Self type and derefs of conversion method `n` on a receiver
    /// of type `rt` in rustc's probe order: at each step the by-value form
    /// (the step is `&X`, or `&mut X` for a `&mut self` method) with X as
    /// Self, then the step itself by autoref. Core's reflexive `Borrow<T>
    /// for T` holds at the first step, so `borrow` takes `X` from a `&X`
    /// receiver and every other receiver as is (`&mut String` borrows as
    /// `&mut String` or `String`, never `str`). `to_owned` holds where
    /// Self is Clone (every shared reference), `str`, or a Clone slice:
    /// `(&&s).to_owned()` is `&String`, `(&mut &s).to_owned()` is
    /// `String`. `as_ref`/`as_mut` peel one reference (their `&T` impls
    /// forward to T's). None when no step holds.
    fn conv_self_step(&mut self, e: ExprId, rt: Ty, n: &str) -> Option<(Ty, u8)> {
        let (d0, start) = self.probe_start(e, rt);
        let ref_parts = |cx: &mut Self, t: Ty| match cx.tys.kind(t) {
            TyKind::Ref(_, m, inner) => Some((m.is_mut(), cx.deep(inner))),
            _ => None,
        };
        match n {
            "borrow" | "borrow_mut" => Some(match ref_parts(self, start) {
                Some((m, x)) if m == (n == "borrow_mut") => (x, d0 + 1),
                _ => (start, d0),
            }),
            "to_owned" => {
                let p = self.p;
                let holds = |cx: &Self, t: Ty| match cx.tys.kind(t) {
                    TyKind::Str => true,
                    TyKind::Slice(el) => has_clone(p, &cx.tys, el),
                    _ => has_clone(p, &cx.tys, t),
                };
                let (mut d, mut t) = (d0, start);
                loop {
                    let r = ref_parts(self, t);
                    if let Some((false, x)) = r
                        && holds(self, x)
                    {
                        return Some((x, d + 1));
                    }
                    if holds(self, t) {
                        return Some((t, d));
                    }
                    t = r?.1;
                    d = d.checked_add(1)?;
                }
            }
            _ => Some(match ref_parts(self, rt) {
                Some((_, x)) => (x, 1),
                None => (rt, 0),
            }),
        }
    }

    /// `x.deref()` and `x.deref_mut()` on a builtin Self (see
    /// `builtin_deref_target`): std's impls return `&**self`, which is
    /// the conversion builtin of `&Self` to `&Target`. Self is the template's pick in an instance, else
    /// rustc's probe order: at each autoderef step a `&X` (`&mut X`)
    /// value whose X derefs, then the step itself by autoref. A
    /// mutable call through a shared reference, `Rc`, or `Arc` is
    /// left to the other dispatch (rustc rejects it: E0596). User
    /// Adts resolve their own impls in the generic method search.
    pub(super) fn deref_call(&mut self, e: ExprId, recv: ExprId, rt: Ty, mutable: bool, name_span: Span) -> Option<Ty> {
        let lang = self.p.gx.lang;
        let tr = if mutable { lang.deref_mut } else { lang.deref };
        let replayed = self.replay_pick(e).filter(|pk| pk.trait_def == tr);
        // A bound brings `Deref` in only for its own param, which is not a
        // builtin Self (an instance counts its template's bounds).
        let in_scope = super::method::trait_in_scope_pub(self, tr) || self.enclosing_impl_trait_pub() == Some(tr);
        if replayed.is_none() && !in_scope {
            return None;
        }
        let (self_ty, derefs) = match replayed {
            Some(pick) => {
                let mut t = rt;
                for _ in 0..pick.derefs {
                    t = match self.tys.kind(t) {
                        TyKind::Ref(_, _, inner) => self.deep(inner),
                        _ => self.deep(self.tys.smart_elem(t)?),
                    };
                }
                (t, pick.derefs)
            }
            None => self.deref_probe(rt, mutable)?,
        };
        let target = self.builtin_deref_target(self_ty, mutable)?;
        // A Cow derefs through its view builtin; a guard's `ConvDeref`
        // lowers as a reborrow of its target place (lower.rs).
        let b = if crate::check::collect::cow_arg(&self.tys, self_ty).is_some() { Builtin::CowDeref } else { self.conv_builtin(self_ty, target) };
        let autoref = if mutable { Autoref::RefMut } else { Autoref::Ref };
        self.check_packed_borrow(recv, name_span);
        self.set_res(e, Res::BuiltinMethod { b, derefs, autoref });
        Some(self.tys.mk_ref(mutable, target))
    }

    /// The Self type and builtin derefs of a `deref`/`deref_mut` call
    /// on a receiver of type `rt` (see `deref_call`), in rustc's probe
    /// order. None when the first step that could hit is a user Adt, a
    /// parameter, or another type whose impls the generic method search
    /// answers (`(&w).deref()` is the user's `deref`; `(&&w).deref()` is
    /// core's `&T` impl, with `&W` as Self).
    fn deref_probe(&mut self, rt: Ty, mutable: bool) -> Option<(Ty, u8)> {
        let deferred = |cx: &Self, t: Ty| {
            matches!(cx.tys.kind(t), TyKind::Param(_) | TyKind::Projection(..) | TyKind::Opaque(..) | TyKind::Dyn(..) | TyKind::Infer(_))
                || (cx.tys.local_adt(t).is_some() && crate::check::collect::cow_arg(&cx.tys, t).is_none())
        };
        let mut t = rt;
        for d in 0..8u8 {
            // The by-value form: `deref(&self)` takes a `&X` step (not a
            // `&mut X`: `(&mut s).deref()` is core's `&mut T` impl, `&String`),
            // `deref_mut` a `&mut X` step.
            if let TyKind::Ref(_, m, x) = self.tys.kind(t)
                && m.is_mut() == mutable
            {
                let x = self.deep(x);
                if deferred(self, x) {
                    return None;
                }
                if self.builtin_deref_target(x, mutable).is_some() {
                    return Some((x, d + 1));
                }
            }
            if deferred(self, t) {
                return None;
            }
            if self.builtin_deref_target(t, mutable).is_some() {
                return Some((t, d));
            }
            t = match self.tys.kind(t) {
                TyKind::Ref(_, m, x) if !mutable || m.is_mut() => self.deep(x),
                _ if self.tys.boxed_elem(t).is_some() => self.deep(self.tys.boxed_elem(t)?),
                _ if !mutable && self.tys.rc_elem(t).is_some() => self.deep(self.tys.rc_elem(t)?),
                _ => match self.tys.guard_elem(t) {
                    Some((el, m)) if !mutable || m => self.deep(el),
                    _ => return None,
                },
            };
        }
        None
    }

    /// The `Deref::Target` of a builtin type with a core or std
    /// `Deref` impl (`DerefMut` when `mutable`): String, Vec, Box,
    /// Rc, Arc, references, `Cow`, and the `RefCell` and lock guards.
    /// None for every other type, user Adts included.
    fn builtin_deref_target(&mut self, t: Ty, mutable: bool) -> Option<Ty> {
        if let Some(b) = crate::check::collect::cow_arg(&self.tys, t) {
            return (!mutable).then(|| self.deep(b));
        }
        if self.tys.local_adt(t).is_some() {
            return None;
        }
        if let Some((el, m)) = self.tys.guard_elem(t) {
            return (!mutable || m).then(|| self.deep(el));
        }
        match self.tys.kind(t) {
            TyKind::Ref(_, m, inner) if !mutable || m.is_mut() => return Some(self.deep(inner)),
            TyKind::Ref(..) => return None,
            _ => {}
        }
        if self.tys.is_string_ty(t) {
            return Some(STR);
        }
        if let Some(el) = self.tys.vec_elem(t) {
            return Some(self.tys.intern(TyKind::Slice(el)));
        }
        if let Some(el) = self.tys.boxed_elem(t) {
            return Some(self.deep(el));
        }
        match self.tys.rc_elem(t) {
            Some(el) if !mutable => Some(self.deep(el)),
            _ => None,
        }
    }

    /// The builtin a synthetic conversion impl runs for a deferred
    /// call (`Borrow::borrow(x)` and kin, selected at body end in
    /// method.rs and traitpath.rs): None when `core` has no impl of
    /// the trait for `target` in the conversion table.
    pub(super) fn conv_call_builtin(&mut self, trait_def: DefId, core: Ty, target: Ty) -> Option<Builtin> {
        let lang = self.p.gx.lang;
        let conv = match () {
            _ if trait_def == lang.as_ref => Conv::AsRef,
            _ if trait_def == lang.as_mut => Conv::AsMut,
            _ if trait_def == lang.borrow => Conv::Borrow,
            _ if trait_def == lang.borrow_mut => Conv::BorrowMut,
            _ => return None,
        };
        let cands = self.conv_targets(conv, core);
        cands.iter().copied().find(|&c| c == target).or_else(|| cands.iter().copied().find(|&c| self.unify(c, target)))?;
        Some(self.conv_builtin(core, target))
    }

    /// The trait arguments of a conversion trait's impls for `core`.
    fn conv_targets(&mut self, conv: Conv, core: Ty) -> Vec<Ty> {
        let bytes = self.tys.intern(TyKind::Slice(int(IntTy::U8)));
        let k = self.tys.kind(core);
        let mut out: Vec<Ty> = Vec::new();
        // Borrow and BorrowMut: every type borrows as itself.
        if matches!(conv, Conv::Borrow | Conv::BorrowMut) {
            out.push(core);
        }
        let elem_slice = |cx: &mut Self, e: Ty| cx.tys.intern(TyKind::Slice(e));
        match (conv, k) {
            (Conv::AsRef, TyKind::Str) => out.extend([STR, bytes]),
            (Conv::AsMut, TyKind::Str) => out.push(STR),
            (Conv::AsRef | Conv::AsMut, TyKind::Slice(_)) => out.push(core),
            (_, TyKind::Array(e, _)) => {
                let s = elem_slice(self, e);
                out.push(s);
            }
            (Conv::Borrow, TyKind::Ref(_, _, inner)) => out.push(inner),
            (Conv::BorrowMut, TyKind::Ref(_, m, inner)) if m.is_mut() => out.push(inner),
            _ if self.tys.is_string_ty(core) => match conv {
                Conv::AsRef => out.extend([STR, bytes]),
                _ => out.push(STR),
            },
            _ => {
                if let Some(e) = self.tys.vec_elem(core) {
                    let s = elem_slice(self, e);
                    out.push(s);
                    if matches!(conv, Conv::AsRef | Conv::AsMut) {
                        out.push(core);
                    }
                } else if let Some(e) = self.tys.boxed_elem(core) {
                    out.push(e);
                } else if let Some(e) = self.tys.rc_elem(core)
                    && matches!(conv, Conv::AsRef | Conv::Borrow)
                {
                    out.push(e);
                }
            }
        }
        out
    }

    /// The builtin that converts `&core` to `&target`.
    fn conv_builtin(&self, core: Ty, target: Ty) -> Builtin {
        if core == target {
            return Builtin::ConvIdent;
        }
        match self.tys.kind(core) {
            TyKind::Str => Builtin::ConvStrBytes,
            TyKind::Array(..) => Builtin::ConvArraySlice,
            TyKind::Ref(..) => Builtin::ConvDeref,
            _ if self.tys.is_string_ty(core) => {
                if target == STR {
                    Builtin::StringAsStr
                } else {
                    Builtin::StringAsBytes
                }
            }
            _ if self.tys.vec_elem(core).is_some() => Builtin::VecAsSlice,
            // An Rc or Arc of an unsized value is its fat value (check/own.rs).
            _ if self.tys.rc_elem(core).is_some() && crate::check::own::own_parts(&self.tys, core).is_none() => Builtin::ConvRcInner,
            _ => Builtin::ConvDeref,
        }
    }

    /// `to_owned()`: `str` to String, `[T]` to `Vec<T>`, and `T: Clone`
    /// to a clone of Self (which can be a reference: `(&&s).to_owned()`
    /// is `&String`).
    fn to_owned_call(&mut self, e: ExprId, recv: ExprId, rt: Ty, self_ty: Ty, self_derefs: u8, name_span: Span) -> Option<Ty> {
        let _ = rt;
        let (b, ret) = match self.tys.kind(self_ty) {
            TyKind::Str => (Builtin::StrToString, STRING),
            TyKind::Slice(el) => {
                if !has_clone(self.p, &self.tys, el) {
                    return None;
                }
                (Builtin::SliceToVec, self.tys.mk_vec(el))
            }
            _ => {
                if !has_clone(self.p, &self.tys, self_ty) {
                    return None;
                }
                (Builtin::Clone, self_ty)
            }
        };
        self.check_packed_borrow(recv, name_span);
        self.set_res(e, Res::BuiltinMethod { b, derefs: self_derefs, autoref: Autoref::Ref });
        Some(ret)
    }

    /// `clone_into(&mut target)`: `str` into a String and a slice of
    /// Copy elements into a Vec, clearing the target first (its
    /// buffer is reused, as std's impls do). A slice of non-Copy
    /// elements clones element by element through each element's
    /// `clone_from`, which lust has no form for yet (L0640); every
    /// other receiver resolves its written or generic impl in the
    /// generic method search (None here).
    fn clone_into_call(&mut self, e: ExprId, recv: ExprId, self_ty: Ty, self_derefs: u8, name_span: Span, args: &[ExprId]) -> Option<Ty> {
        let owned = match self.tys.kind(self_ty) {
            TyKind::Str => STRING,
            TyKind::Slice(el) => {
                if !is_copy(self.p, &self.tys, el) {
                    self.err(
                        "L0640",
                        name_span,
                        "`clone_into` on a slice of non-Copy elements is not implemented yet".to_string(),
                        "Assign a fresh Vec instead: `*target = src.to_vec()`.",
                    );
                    self.expr(args[0], None);
                    return Some(ERROR);
                }
                self.tys.mk_vec(el)
            }
            _ => return None,
        };
        self.check_packed_borrow(recv, name_span);
        self.set_res(e, Res::BuiltinMethod { b: Builtin::CloneInto, derefs: self_derefs, autoref: Autoref::Ref });
        let target = self.tys.mk_ref(true, owned);
        self.check_args(&[target], args, name_span);
        Some(UNIT)
    }

    /// In an instance of a generic body, the trait argument of the
    /// template's bound `self_ty: trait_def<U>` (substituted at the
    /// instance's args): what the template's call through that bound
    /// meant, which a builtin Self type cannot tell by its impls alone.
    pub(super) fn instance_bound_arg(&mut self, trait_def: DefId, self_ty: Ty) -> Option<Ty> {
        let p = self.p;
        let info = p.gx.instances.get(&self.fid)?;
        let tl = info.template_local?;
        let env = p.gx.fn_scopes.get(tl.0 as usize)?.env.clone();
        let args: Vec<GenericArg> = p.tys.args(info.args).to_vec();
        for pred in env {
            let Predicate::Trait(b) = pred else { continue };
            if b.value.def != trait_def || b.count != 0 {
                continue;
            }
            let tr = crate::traits::index::subst_trait_ref_pub(&mut self.tys, b.value, &args);
            // A bound on a projection (`I::Item: AsRef<str>`) names its
            // Self normalized in the instance (check/project.rs).
            let a: Vec<Ty> = self.tys.args(tr.args).to_vec().into_iter().filter_map(|g| g.as_ty()).collect();
            let a: Vec<Ty> = a.into_iter().map(|t| self.normalize_here(t)).collect();
            if a.first().copied() == Some(self_ty) {
                return a.get(1).copied();
            }
        }
        None
    }

    /// `x.into()` through one of std's `From` impls between builtin
    /// types (`std_from`). Self is the receiver itself: core's blanket
    /// `impl<T, U: From<T>> Into<U> for T` matches at the first
    /// autoderef step, so `(&s).into()` converts the reference (an
    /// unsized place autorefs: `v[1..].into()` converts `&v[1..]`). The
    /// target is the expected type, else (in an instance of a generic
    /// body) the template's `Into<U>` bound. None for other calls.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn std_into_method(
        &mut self,
        e: ExprId,
        rt: Ty,
        name: Sym,
        args: &[ExprId],
        turbofish: &[ast::GenericArg],
        expected: Option<Ty>,
    ) -> Option<Ty> {
        if !turbofish.is_empty() || self.p.interner.get(name) != "into" || !args.is_empty() {
            return None;
        }
        let (src, autoref) = self.into_self(rt);
        // Fully known: no inference variable at all (an integer or float
        // literal's type too) and no error.
        let known = |cx: &Self, t: Ty| !cx.tys.flags(t).contains_infer() && !cx.has_error_kind(t);
        let dst = match expected.map(|t| self.deep(t)) {
            Some(t) if known(self, t) => t,
            _ => self.instance_bound_arg(self.p.gx.lang.into, src)?,
        };
        if !known(self, dst) {
            return None;
        }
        // A literal's integer or float type comes from the target, as
        // the one std impl that can apply fixes it in rustc (`5.into()`
        // as an `Option<i32>`, `[1, 2].into()` as a `Vec<i32>`).
        let src = if known(self, src) { src } else { self.std_from_literal_src(dst, src)? };
        self.std_from_here(dst, src)?;
        self.set_res(e, Res::BuiltinMethod { b: Builtin::ConvInto, derefs: 0, autoref });
        Some(dst)
    }

    /// `std_into_method`'s source when it still has an integer or float
    /// variable: the variable unified with the element the target fixes
    /// (`Option<T>`, `Box<T>`, `Rc<T>` from T; `Rc<T>`, `Arc<T>` from
    /// `Box<T>`; `Vec<T>`, `Box<[T]>`,
    /// `Rc<[T]>`, `Arc<[T]>`, `Cow<[T]>` from `[T; N]`, `&[T; N]`,
    /// `&[T]`, `Vec<T>`). None for any other
    /// shape, and when the unification fails (nothing stays bound).
    pub(super) fn std_from_literal_src(&mut self, dst: Ty, src: Ty) -> Option<Ty> {
        let lit = |cx: &Self, t: Ty| matches!(cx.kind(t), k if k.is_int_infer() || k.is_float_infer());
        let elem_of_seq = |cx: &Self, t: Ty| -> Option<Ty> {
            let t = match cx.kind(t) {
                TyKind::Ref(_, _, i) => cx.resolve(i),
                _ => t,
            };
            match cx.kind(t) {
                TyKind::Array(e, _) | TyKind::Slice(e) => Some(cx.resolve(e)),
                _ => cx.tys.vec_elem(t).map(|e| cx.resolve(e)),
            }
        };
        let boxed_lit = self.tys.boxed_elem(src).map(|b| self.resolve(b)).filter(|&b| lit(self, b));
        let (have, want) = if lit(self, src) {
            (src, self.tys.option_elem(dst).or_else(|| self.tys.boxed_elem(dst)).or_else(|| self.tys.rc_elem(dst))?)
        } else if let Some(b) = boxed_lit {
            // `Box::new(5).into()` as an `Rc<i32>` (alloc's `From<Box<T>>`).
            (b, self.tys.rc_elem(dst)?)
        } else {
            // An element with a literal anywhere (`({integer}, String)`).
            let e = elem_of_seq(self, src).filter(|&e| lit(self, e) || self.tys.flags(e).contains_infer())?;
            let want = self
                .tys
                .vec_elem(dst)
                .or_else(|| self.tys.boxed_slice_elem(dst))
                .or_else(|| crate::check::own::shared_unsized_elem(&self.tys, dst).and_then(|b| elem_of_seq(self, b)))
                .or_else(|| crate::check::collect::cow_arg(&self.tys, dst).and_then(|b| elem_of_seq(self, b)))?;
            (e, want)
        };
        let snap = self.snapshot_vars();
        if !self.unify(have, want) {
            self.rollback_vars(snap);
            return None;
        }
        let src = self.deep(src);
        if std_from(self.p, &self.tys, dst, src).is_none() {
            self.rollback_vars(snap);
            return None;
        }
        Some(src)
    }

    /// The argument type of a `T::from(x)` call that `std_from` answers
    /// for the target `dst`: `at` resolved, when fully known.
    pub(super) fn std_from_arg(&mut self, dst: Ty, at: Ty) -> Option<Ty> {
        // Inference variables are fine here: `std_from` matches by
        // equality, so a variable only matches itself (`Vec::from` of a
        // `Box<[{integer}]>` keeps the literal's variable in both).
        let at = self.deep(at);
        if self.has_error_kind(at) {
            return None;
        }
        self.std_from_here(dst, at).map(|_| at)
    }

    /// `std_from` with the checker's normalization for the one shape
    /// structure cannot decide: `Cow<B>` from `<B as ToOwned>::Owned`
    /// where `B`'s `ToOwned` is a written impl (Path, OsStr, a user
    /// unsized type). The projection must normalize to `src` here;
    /// at lowering the mono instance's concrete Owned field answers
    /// the same question in `std_from`.
    fn std_from_here(&mut self, dst: Ty, src: Ty) -> Option<FromConv> {
        if let Some(f) = std_from(self.p, &self.tys, dst, src) {
            return Some(f);
        }
        let b = crate::check::collect::cow_arg(&self.tys, dst)?;
        let a = self.tys.intern_args(&[GenericArg::Ty(b)]);
        let pr = self.tys.intern(TyKind::Projection(AssocId(self.p.gx.lang.assoc_owned), a));
        let owned = self.normalize_here(pr);
        (owned != pr && owned == src).then_some(FromConv::CowOwned)
    }
}

/// True when the crate has a written impl of `trait_def` for the Adt
/// `def` (any args): `impl Borrow<str> for P`.
fn user_impl_for_adt(p: &crate::check::Program, trait_def: DefId, def: DefId) -> bool {
    use crate::traits::index::{SelfHead, Synthetic};
    p.gx.index.impls_of_trait(trait_def).iter().any(|&id| {
        let rec = p.gx.index.peek(id);
        rec.synthetic == Synthetic::None && rec.head == SelfHead::Adt(def)
    })
}
