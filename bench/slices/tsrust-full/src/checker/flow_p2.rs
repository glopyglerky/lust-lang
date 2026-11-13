//! Port of typescript-go `internal/checker/flow.go` lines 953-1854
//! (`getInstanceType` through `getReferenceCandidate`).
//!
//! PORT: Go `*FlowState` is `Rc<RefCell<FlowState>>` (see `Checker::free_flow_state`),
//! so flow functions take `f: &Rc<RefCell<FlowState>>`. Borrows of `f` are never
//! held across calls into the checker.

use crate::prelude::*;
use std::borrow::Cow;
use std::sync::LazyLock;


// Go: checker/flow.go:1496 getCandidateVariableDeclarationInitializer



// Go: checker/flow.go:1651 nonDottedNameCacheKey
// PORT: Go package var `CacheHashKey(xxh3.HashString128("?"))`, computed through
// `KeyBuilder` like `SIGNATURE_KEY_*` in checker_p01. Read with `*NON_DOTTED_NAME_CACHE_KEY`.


// Go: checker/flow.go:1782 tryGetNameFromType
// PORT: Go package function that reads type data. The contract makes such
// functions `Checker` methods, but `Checker::try_get_name_from_type` already
// exists (Go method `(*Checker).tryGetNameFromType`, checker.go:18621, which
// behaves differently). This stays a free function that takes the checker.

