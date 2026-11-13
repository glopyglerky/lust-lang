//! Port of `checker/checker.go` lines 19436-20350: base types, declared
//! members, index infos of symbols, signatures from declarations, late-bound
//! names, return types of signatures and bodies, and promise/generator types.

use crate::prelude::*;
use crate::smallvec::SmallVec;

// PORT: Go `core.AppendIfUnique` for type slices.
fn append_if_unique_p22(list: &mut Vec<TypeId>, t: TypeId) {
    if !list.contains(&t) {
        list.push(t);
    }
}

// PORT: Go `core.OrElse` for types.
fn or_else_type_p22(t: TypeId, fallback: TypeId) -> TypeId {
    if t.is_some() { t } else { fallback }
}


// Go: checker/checker.go:20327 isLateBindableAST


// Go: checker/checker.go:20455 getEffectiveSetAccessorTypeAnnotationNode


// Go: checker/checker.go:20649 mayReturnNever

