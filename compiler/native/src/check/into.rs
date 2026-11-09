// `x.into()` through core's blanket `impl<T, U: From<T>> Into<U> for T`
// when the `From` impl is one of std's that lust keeps builtin: the
// reflexive `From<T> for T`, `String` from `&str`, `&String`, and
// `char`, and `Box<T>` and `Rc<T>` from `T`. The selection facts are
// the synthetic records in generics.rs (`synthetic_lang_impls`); this
// is the call side, after the user impl search found no `from` body.
//
// Self is the receiver as written (the blanket matches at the first
// autoderef step, by value; an unsized place autorefs, see `into_self`).
// The target comes from the template's `T: Into<U>` evidence in an
// instance of a generic body, else from the expected type; with neither
// the call stays unresolved (rustc E0282).
use super::*;

impl<'p> Ctx<'p> {
    /// `into()` to a target std converts the receiver to without a
    /// user body. None when the call is not one of these.
    pub(super) fn builtin_into_method(
        &mut self,
        e: ExprId,
        rt: Ty,
        name: Sym,
        args: &[ExprId],
        turbofish: &[ast::GenericArg],
        expected: Option<Ty>,
    ) -> Option<Ty> {
        if !turbofish.is_empty() || !args.is_empty() || self.name(name) != "into" {
            return None;
        }
        // An unsized receiver place autorefs (`into_self`); the
        // `&[T]`/`&str` forms are `std_into_method`'s.
        if self.into_self(rt).1 != Autoref::None {
            return None;
        }
        let into = self.p.gx.lang.into;
        let replayed = self.replayed_trait_ref(e, into).and_then(|r| self.tys.args(r.args).get(1).and_then(|g| g.as_ty()));
        let dst = self.deep(replayed.or(expected)?);
        let src = self.deep(rt);
        if matches!(self.kind(src), TyKind::Infer(_) | TyKind::Error) || matches!(self.kind(dst), TyKind::Infer(_) | TyKind::Error) {
            return None;
        }
        let res = if self.tys.is_string_ty(dst) {
            match self.kind(src) {
                TyKind::Char => Res::BuiltinMethod { b: Builtin::StringFrom, derefs: 0, autoref: Autoref::None },
                TyKind::Ref(_, Mut::Shared, inner) if self.resolve(inner) == STR => {
                    Res::BuiltinMethod { b: Builtin::StringFrom, derefs: 0, autoref: Autoref::None }
                }
                // `impl From<&String> for String` clones.
                TyKind::Ref(_, Mut::Shared, inner) if self.tys.is_string_ty(self.resolve(inner)) => {
                    Res::BuiltinMethod { b: Builtin::Clone, derefs: 1, autoref: Autoref::Ref }
                }
                _ if src == dst => Res::PrimOp { kind: PrimOpKind::IntoIterSelf, derefs: 0 },
                _ => return None,
            }
        } else if let Some(elem) = self.tys.boxed_elem(dst).filter(|_| self.tys.boxed_slice_elem(dst).is_none())
            && self.into_unifies(elem, src)
        {
            Res::BuiltinMethod { b: Builtin::BoxNew, derefs: 0, autoref: Autoref::None }
        } else if let Some(elem) = self.tys.rc_elem(dst)
            && self.into_unifies(elem, src)
        {
            Res::BuiltinMethod { b: Builtin::RcNew, derefs: 0, autoref: Autoref::None }
        } else if self.into_unifies(dst, src) {
            // `From<T> for T`: the value moves through.
            Res::PrimOp { kind: PrimOpKind::IntoIterSelf, derefs: 0 }
        } else {
            return None;
        };
        self.set_res(e, res);
        Some(dst)
    }

    /// Self of `x.into()` on a receiver of type `rt`, and the autoref
    /// that makes it. Core's blanket `Into` impl needs `Self: Sized`, so
    /// on an unsized place (`v[1..]`, `*s`, `*boxed`) rustc's probe skips
    /// the by-value step and takes the first autoref step: Self is
    /// `&[T]` or `&str`, never the unsized value.
    pub(super) fn into_self(&mut self, rt: Ty) -> (Ty, Autoref) {
        let t = self.deep(rt);
        if matches!(self.kind(t), TyKind::Infer(_) | TyKind::Error) || self.sized_ok(t) {
            return (t, Autoref::None);
        }
        (self.tys.mk_ref(false, t), Autoref::Ref)
    }

    /// Unifies two types for an `into` impl match, rolling back on a miss.
    fn into_unifies(&mut self, a: Ty, b: Ty) -> bool {
        let snap = self.snapshot_vars();
        let ok = self.unify(a, b);
        if !ok {
            self.rollback_vars(snap);
        }
        ok
    }
}
