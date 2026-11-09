//! Port of Effect-TS/tsgo `internal/rules/try_catch_in_effect_gen.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

// TryCatchInEffectGen detects try/catch statements inside Effect generators
// and suggests using Effect's error handling mechanisms instead.


// checkTryCatchScope checks if the try statement is directly inside an Effect
// generator scope using FindEnclosingScopes.
