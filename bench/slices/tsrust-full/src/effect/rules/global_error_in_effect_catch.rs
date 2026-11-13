//! Port of Effect-TS/tsgo `internal/rules/global_error_in_effect_catch.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

/// Go `globalErrorCatchApis`: the Effect module APIs that have a catch callback parameter.
static GLOBAL_ERROR_CATCH_APIS: &[&str] = &["tryPromise", "try", "tryMap", "tryMapPromise"];

// Go: rules.GlobalErrorInEffectCatch



// Go: rules.checkGlobalErrorInEffectCatch

// Go: rules.isGlobalErrorCatchCallee
