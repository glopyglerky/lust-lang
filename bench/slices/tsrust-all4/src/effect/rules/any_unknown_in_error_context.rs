//! Port of Effect-TS/tsgo `internal/rules/any_unknown_in_error_context.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;


struct MatchEntry {
    node: Node,
    message_text: String,
    start: i32,
    end: i32,
}

