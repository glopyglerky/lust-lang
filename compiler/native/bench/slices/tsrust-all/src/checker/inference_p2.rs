//! Port of typescript-go `checker/inference.go` lines 947-1627.
//!
//! PORT: Go `*InferenceInfo` is an index into
//! `inference_contexts[ctx].inferences` (see `PORTING.md`). Go functions that
//! take a standalone `*InferenceInfo` take `(ctx: InferenceContextId,
//! index: usize)`. Go functions that take `[]*InferenceInfo` take the
//! `InferenceContextId` that owns the list.

use crate::prelude::*;
use crate::smallvec::SmallVec;


// Go: checker/inference.go:1626 newInferenceInfo
pub fn new_inference_info(type_parameter: TypeId) -> InferenceInfo {
    InferenceInfo {
        type_parameter,
        priority: InferencePriority::MAX_VALUE,
        top_level: true,
        implied_arity: -1,
        ..InferenceInfo::default()
    }
}

// Go: checker/inference.go:1630 cloneInferenceInfo
pub fn clone_inference_info(info: &InferenceInfo) -> InferenceInfo {
    InferenceInfo {
        type_parameter: info.type_parameter,
        candidate_lists: info.candidate_lists.clone(),
        inferred_type: info.inferred_type,
        priority: info.priority,
        top_level: info.top_level,
        is_fixed: info.is_fixed,
        implied_arity: info.implied_arity,
    }
}


// PORT: Go package function `hasTypeParameterDefault` in inference.go has the
// same name as the method `(*Checker).hasTypeParameterDefault`
// (checker.go:21847, `Checker::has_type_parameter_default`). It is a free
// function that takes the checker for type and symbol data, so the two
// names do not collide.
// Go: checker/inference.go:1659 hasTypeParameterDefault

