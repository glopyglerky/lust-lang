//! Port of Go `checker/checker.go` lines 7528-8444 (checkExpression through
//! resolveSignature).

use crate::prelude::*;

// PORT: Go `hasInferenceCandidates(info)` (inference.go) inlined as a local
// helper on the Rust `InferenceInfo` value, because the Go `[]*InferenceInfo`
// slices used here are not all owned by one inference context.
fn info_has_inference_candidates(info: &InferenceInfo) -> bool {
    !info.candidates().is_empty() || !info.contra_candidates().is_empty()
}


// Go: checker/checker.go:8289 isSpreadIntoCallOrNew

