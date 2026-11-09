// Sized well-formedness (rustc E0277, ARCH 2.7 `Sized` basics).
//
// Two questions, one answer each. `parts` names the positions inside a
// type that need a `Sized` type for the type to be well-formed: tuple
// elements but the last, array and slice elements, the type args of the
// builtin Adts other than `Box`, `Rc`, `Arc`, `RefCell`, its guards, and `Mutex`. Local Adts owe the implicit
// `Sized` of their params through their predicates, which the WF walks
// already prove. `is_sized` answers for one type under a param env.
//
// Callers own the walk and the value positions:
// - collect (`check_written_wf`): every signature, field list, impl
//   header, and where clause, plus the value positions there (params
//   and return of a fn with a body, by-value `self` included, struct
//   fields but the last, every enum field);
// - the checker (`wf_check_written`, `let`, closures): written body
//   types, `let` bindings with a written type, closure params and
//   return types.
use crate::check::collect::Program;
use crate::ty::*;

/// Calls `need` for each direct part of `t` that must be `Sized` for
/// `t` to be well-formed. The caller walks into the parts itself.
pub fn parts(tys: &impl TyIntern, t: Ty, need: &mut impl FnMut(Ty)) {
    match tys.kind(t) {
        TyKind::Tuple(ts) => {
            let elems = tys.tys_of(ts);
            for &e in elems.iter().take(elems.len().saturating_sub(1)) {
                need(e);
            }
        }
        TyKind::Array(e, _) | TyKind::Slice(e) => need(e),
        TyKind::Adt(_, a) => match tys.builtin_of(t) {
            None | Some(BuiltinTy::Boxed | BuiltinTy::Rc | BuiltinTy::Weak | BuiltinTy::Sync(crate::ty::sync::SyncTy::Arc | crate::ty::sync::SyncTy::ArcWeak)) => {}
            // `RefCell<T: ?Sized>` and `Mutex<T: ?Sized>`: with a dyn
            // value they are unsized themselves; `MutexGuard<T: ?Sized>`
            // and the `RefCell` guards `Ref`/`RefMut<T: ?Sized>` are
            // pointers.
            Some(
                BuiltinTy::RefCell
                | BuiltinTy::RefGuard
                | BuiltinTy::RefMutGuard
                | BuiltinTy::Sync(crate::ty::sync::SyncTy::Mutex | crate::ty::sync::SyncTy::MutexGuard),
            ) => {}
            Some(_) => {
                for g in tys.args(a) {
                    if let Some(e) = g.as_ty() {
                        need(e);
                    }
                }
            }
        },
        _ => {}
    }
}

/// True when `t` is `Sized`. `str`, slices, and dyn types are not; a
/// param is when `param_sized` says so; a tuple is when its last
/// element is, and a struct when its last field is (under the written
/// args). Enums, builtin Adts, projections, and inference variables
/// count as sized (a projection is normalized before layout).
pub fn is_sized(p: &Program, tys: &mut impl TyIntern, t: Ty, param_sized: &dyn Fn(ParamId) -> bool) -> bool {
    is_sized_depth(p, tys, t, param_sized, 0)
}

fn is_sized_depth(p: &Program, tys: &mut impl TyIntern, t: Ty, param_sized: &dyn Fn(ParamId) -> bool, depth: u32) -> bool {
    match tys.kind(t) {
        TyKind::Str | TyKind::Slice(_) | TyKind::Dyn(..) => false,
        TyKind::Adt(..) if crate::check::own::dyn_wrapped_elem(tys, t).is_some() => false,
        TyKind::Param(q) => param_sized(q),
        TyKind::Tuple(ts) => match tys.tys_of(ts).last() {
            Some(&e) => is_sized_depth(p, tys, e, param_sized, depth),
            None => true,
        },
        // A struct is as sized as its last field. The depth bound stops
        // a field cycle the infinite-size check reports on its own.
        TyKind::Adt(d, a) if depth < 16 && tys.builtin_of(t).is_none() => {
            let Some(id) = p.adt_by_def(d) else { return true };
            let adt = p.adt(id);
            if adt.is_enum {
                return true;
            }
            let Some(last) = adt.variants.first().and_then(|v| v.fields.last()).map(|f| f.ty) else { return true };
            let args: Vec<GenericArg> = tys.args(a).to_vec();
            let last = if args.is_empty() { last } else { subst(tys, last, &args) };
            is_sized_depth(p, tys, last, param_sized, depth + 1)
        }
        _ => true,
    }
}

/// The params `env` holds a `Sized` predicate for. Read the env
/// in the caller's own store once, before `is_sized` needs the
/// store mutably.
pub fn env_sized_params(tys: &impl TyIntern, env: &[Predicate], sized: crate::def::DefId) -> Vec<ParamId> {
    let mut out: Vec<ParamId> = Vec::new();
    for pr in env {
        if let Predicate::Trait(b) = pr
            && b.value.def == sized
            && let Some(s) = tys.args(b.value.args).first().and_then(|g| g.as_ty())
            && let TyKind::Param(r) = tys.kind(s)
        {
            out.push(r);
        }
    }
    out
}

/// The L0110 diagnostic for a type that must be `Sized` and is not.
pub fn unsized_diag(p: &Program, tys: &impl TyIntern, t: Ty, span: crate::diag::Span) -> crate::diag::Diag {
    crate::diag::Diag::new(
        "L0110",
        span,
        format!("the size for values of type `{}` cannot be known", crate::check::collect::ty_name(p, tys, t)),
        "Put the value behind a reference such as `&T`, or add a `Sized` bound.",
    )
}
