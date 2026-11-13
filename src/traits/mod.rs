// The trait system (docs/ARCH.md sections 2.3 to 2.6, W-TRAITS-CORE).
//
// `index` holds the impl and trait records built at collect plus the
// coherence and orphan checks. `solve` holds selection (section 2.4),
// the provably-false check, and normalization of projections (2.5).
// Nothing here knows about bodies or the checker: selection runs over
// the small `SolveCx` interface, which the checker implements over its
// inference variables and mono implements over concrete types.
//
// Convention, used everywhere in this module: a trait's `Generics`
// starts with a synthetic `Self` type parameter at `ParamId(0)`, then
// the written parameters. A `TraitRef`'s args follow the same order
// (Self first), so `subst` with the trait ref's args instantiates any
// signature written inside the trait.
pub mod index;
pub mod solve;

#[allow(unused_imports)] // Re-exported for the checker and mono; first callers land with them.
pub use index::{ImplId, ImplIndex, ImplRecord, SelfHead, TraitRecord, head_of};
#[allow(unused_imports)] // See above.
pub use solve::{Selection, SolveCx, Solver, normalize};

use crate::def::DefId;
use crate::ty::*;

/// True when a type is `Sized` (ARCH 2.7 basics). `param_sized` answers
/// for a generic parameter: true unless the parameter was written
/// `?Sized` (collect records that as the absence of a `Sized`
/// predicate in the param env). `adt_sized` answers for an Adt from
/// its field types, because a struct whose last field is unsized is
/// itself unsized; the caller walks the fields so this module stays
/// free of the Adt table. An inference variable answers true: it can
/// still only bind to a sized type in a sized position, and the
/// checker reports the mismatch at unification.
pub fn is_sized(
    tys: &impl TyIntern,
    t: Ty,
    param_sized: &dyn Fn(ParamId) -> bool,
    adt_sized: &dyn Fn(&dyn Fn(Ty) -> bool, DefId, ArgsId) -> bool,
) -> bool {
    match tys.kind(t) {
        TyKind::Str | TyKind::Slice(_) | TyKind::Dyn(..) => false,
        TyKind::Param(p) => param_sized(p),
        TyKind::Adt(d, _) if tys.unsized_field(d).is_some() => false,
        TyKind::Adt(d, a) => adt_sized(&|e| is_sized(tys, e, param_sized, adt_sized), d, a),
        TyKind::Tuple(ts) => tys.tys_of(ts).last().is_none_or(|&e| is_sized(tys, e, param_sized, adt_sized)),
        // A projection is sized unless proven otherwise; mono
        // normalizes before layout, so a wrong answer here can only
        // come from an unnormalized projection in a sized position,
        // which the obligation for its own trait rejects first.
        _ => true,
    }
}
