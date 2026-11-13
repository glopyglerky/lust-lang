//! Port of Effect-TS/tsgo `internal/rules/timeout_catch_tag_to_timeout_or_else.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;




// Go: rules/timeout_catch_tag_to_timeout_or_else.go AnalyzeTimeoutCatchTagToTimeoutOrElse

// Go: rules/timeout_catch_tag_to_timeout_or_else.go timeoutAsSome

// Go: rules/timeout_catch_tag_to_timeout_or_else.go timeoutReturnsNone

// Fail closed for unknown, generic, or broadly tagged errors: catchTag matches
// structurally by tag, so checking only the canonical TimeoutError is unsafe.
// Go: rules/timeout_catch_tag_to_timeout_or_else.go timeoutErrorExcluded

// Restrict callbacks to arrows so the caught error cannot be observed through
// a function's arguments object. An unused plain parameter can be removed.
// Go: rules/timeout_catch_tag_to_timeout_or_else.go timeoutIgnoresError
