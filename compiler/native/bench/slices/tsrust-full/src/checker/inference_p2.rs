//! Port of typescript-go `checker/inference.go` lines 947-1627.
//!
//! PORT: Go `*InferenceInfo` is an index into
//! `inference_contexts[ctx].inferences` (see `PORTING.md`). Go functions that
//! take a standalone `*InferenceInfo` take `(ctx: InferenceContextId,
//! index: usize)`. Go functions that take `[]*InferenceInfo` take the
//! `InferenceContextId` that owns the list.

use crate::prelude::*;


// Go: checker/inference.go:1626 newInferenceInfo

// Go: checker/inference.go:1630 cloneInferenceInfo


// PORT: Go package function `hasTypeParameterDefault` in inference.go has the
// same name as the method `(*Checker).hasTypeParameterDefault`
// (checker.go:21847, `Checker::has_type_parameter_default`). It is a free
// function that takes the checker for type and symbol data, so the two
// names do not collide.
// Go: checker/inference.go:1659 hasTypeParameterDefault

