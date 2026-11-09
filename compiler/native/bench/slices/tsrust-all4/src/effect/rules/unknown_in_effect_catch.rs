//! Port of Effect-TS/tsgo `internal/rules/unknown_in_effect_catch.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

/// unknownCatchApis lists the Effect module APIs that have a catch callback parameter.
static UNKNOWN_CATCH_APIS: &[&str] = &["tryPromise", "try", "tryMap", "tryMapPromise"];




