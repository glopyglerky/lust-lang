//! Port of Effect-TS/tsgo `internal/rules/effect_in_failure.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;


// Go: rules/effect_in_failure.go stackEntry (local type of the Run closure)
struct StackEntry {
    node: Node,
    visited: bool,
}

