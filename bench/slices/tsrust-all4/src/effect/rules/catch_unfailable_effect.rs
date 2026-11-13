//! Port of Effect-TS/tsgo `internal/rules/catch_unfailable_effect.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

/// Go `catchFunctions`: the Effect module catch functions to check (V3 and V4).
static CATCH_FUNCTIONS: &[&str] = &[
    "catch",
    "catchAll",
    "catchIf",
    "catchSome",
    "catchTag",
    "catchTags",
];

// Go: rules.CatchUnfailableEffect


// Go: rules.isCatchCallee
