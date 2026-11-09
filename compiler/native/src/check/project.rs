// Projection normalization at use sites (docs/ARCH.md section 2.5;
// W-TRAITS).
//
// Collect normalizes every signature and field type once the impl
// index is complete (`generics::normalize_signatures`). A body then
// instantiates those types with its own inference variables, which
// can block a projection: `<?A as Array>::Item` before the receiver
// binds `?A`. Such a projection becomes a fresh variable plus a
// queued `AssocEq` obligation, as rustc normalizes to an inference
// variable; the queue binds the variable at the next run point where
// selection can pick the impl (a coercion site, at the latest the
// body end). A projection with no inference variable left stays as
// written: in a template it is rigid (`<T as Tr>::Out` with no
// binding in the env).
use super::*;

impl<'p> Ctx<'p> {
    /// Substitutes generic args into one signature or field type and
    /// normalizes it at this use (`normalize_use`).
    pub(super) fn subst_norm(&mut self, t: Ty, args: &[GenericArg], span: Span) -> Ty {
        let t = subst(&mut self.tys, t, args);
        self.normalize_use(t, span)
    }

    /// Substitutes and normalizes a signature type where no inference
    /// variable is left in it, binding nothing: `callable_sig` answers
    /// speculative questions too (LUB, coercions), so a blocked
    /// projection stays as it is there.
    pub(super) fn subst_settled(&mut self, t: Ty, args: &[GenericArg]) -> Ty {
        let t = subst(&mut self.tys, t, args);
        let t = self.deep(t);
        if self.tys.flags(t).contains_infer() { t } else { self.normalize_here(t) }
    }

    /// `normalize_use` over the params of an instantiated fn
    /// signature. Callers normalize the return type after checking
    /// the args, which often bind what its projections need.
    pub(super) fn normalize_params(&mut self, params: &mut [Ty], span: Span) {
        for t in params.iter_mut() {
            *t = self.normalize_use(*t, span);
        }
    }

    /// Normalizes one instantiated type here: projections the solver
    /// rewrites now are rewritten, and each one still blocked on an
    /// inference variable becomes a fresh variable with an `AssocEq`
    /// obligation at `span`.
    pub(super) fn normalize_use(&mut self, t: Ty, span: Span) -> Ty {
        if !self.tys.flags(t).contains_projection() {
            return t;
        }
        let t = self.normalize_here(t);
        self.defer_projections(t, span)
    }

    /// Replaces the projections of a normalized type whose trait args
    /// still hold an inference variable (see `normalize_use`).
    fn defer_projections(&mut self, t: Ty, span: Span) -> Ty {
        let flags = self.tys.flags(t);
        if !flags.contains_projection() || !flags.contains_infer() {
            return t;
        }
        match self.tys.kind(t) {
            TyKind::Projection(assoc, trait_args) => {
                let v = self.new_var(false);
                let eq = AssocEqData { assoc: assoc.0, trait_args, ty: v };
                self.obl.push(obligations::Obligation::AssocEq { eq, span });
                v
            }
            TyKind::Ref(r, m, e) => {
                let e = self.defer_projections(e, span);
                self.tys.intern(TyKind::Ref(r, m, e))
            }
            TyKind::RawPtr(m, e) => {
                let e = self.defer_projections(e, span);
                self.tys.intern(TyKind::RawPtr(m, e))
            }
            TyKind::Slice(e) => {
                let e = self.defer_projections(e, span);
                self.tys.intern(TyKind::Slice(e))
            }
            TyKind::Array(e, n) => {
                let e = self.defer_projections(e, span);
                self.tys.intern(TyKind::Array(e, n))
            }
            TyKind::Tuple(ts) => {
                let old: Vec<Ty> = self.tys.tys_of(ts).to_vec();
                let new: Vec<Ty> = old.into_iter().map(|e| self.defer_projections(e, span)).collect();
                let ts = self.tys.intern_tys(&new);
                self.tys.intern(TyKind::Tuple(ts))
            }
            TyKind::Adt(d, a) => {
                let a = self.defer_args(a, span);
                self.tys.intern(TyKind::Adt(d, a))
            }
            TyKind::FnPtr(sig) => {
                let b = self.tys.sig(sig);
                let old: Vec<Ty> = self.tys.tys_of(b.value.inputs).to_vec();
                let new: Vec<Ty> = old.into_iter().map(|e| self.defer_projections(e, span)).collect();
                let inputs = self.tys.intern_tys(&new);
                let output = self.defer_projections(b.value.output, span);
                let sig = self.tys.intern_sig(Binder { count: b.count, value: FnSig { inputs, output, ..b.value } });
                self.tys.intern(TyKind::FnPtr(sig))
            }
            _ => t,
        }
    }

    /// After the body-end zonk: a concrete body (not a template
    /// check) keeps no projection type, since codegen has no C form for
    /// one (ARCH 2.5). One that normalizes now is rewritten; any other
    /// is an error here, never a silent `()` in the generated code.
    pub(super) fn settle_projections(&mut self) {
        if self.is_template_check() {
            return;
        }
        let mut reported = false;
        for i in 0..self.out.expr_ty.len() {
            let t = self.out.expr_ty[i];
            if !self.tys.flags(t).contains_projection() {
                continue;
            }
            let n = self.normalize_here(t);
            if self.tys.flags(n).contains_projection() {
                if !reported {
                    let span = self.span(ExprId(self.out.expr_base + i as u32));
                    self.err(
                        "L0610",
                        span,
                        "type annotations needed: an associated type could not be resolved here",
                        "Annotate the types involved, or name the associated type in the bound.",
                    );
                    reported = true;
                }
                continue;
            }
            self.out.expr_ty[i] = n;
        }
    }

    /// A zonked pattern or local type with its projections normalized
    /// (`let r = x.try_into();` before a later use fixes the target),
    /// as `settle_projections` does for expression types. A type that
    /// does not normalize stays as it is: its expressions report it.
    pub(super) fn settled(&mut self, t: Ty) -> Ty {
        if self.is_template_check() || !self.tys.flags(t).contains_projection() {
            return t;
        }
        let n = self.normalize_here(t);
        if self.tys.flags(n).contains_projection() { t } else { n }
    }

    fn defer_args(&mut self, a: ArgsId, span: Span) -> ArgsId {
        let old: Vec<GenericArg> = self.tys.args(a).to_vec();
        let new: Vec<GenericArg> = old
            .into_iter()
            .map(|g| match g {
                GenericArg::Ty(e) => GenericArg::Ty(self.defer_projections(e, span)),
                other => other,
            })
            .collect();
        self.tys.intern_args(&new)
    }
}
