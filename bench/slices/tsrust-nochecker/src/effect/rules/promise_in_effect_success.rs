//! Port of Effect-TS/tsgo `internal/rules/promise_in_effect_success.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



// Go: rules/promise_in_effect_success.go hasExplicitPromiseEffectContext

// Go: rules/promise_in_effect_success.go isExplicitPromiseEffectType

// Go: rules/promise_in_effect_success.go hasExplicitPromiseEffectReturn

// Go: rules/promise_in_effect_success.go typeContainsPromise

/// explicitPromiseSuccessApis lists the Effect APIs whose explicit type
/// arguments annotate the success channel directly.
static EXPLICIT_PROMISE_SUCCESS_APIS: &[&str] = &["succeed", "as", "map", "zipWith"];



// Go: rules/promise_in_effect_success.go isExplicitPromiseSuccessApi

// Go: rules/promise_in_effect_success.go isEffectSyncCall
