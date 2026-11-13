//! Port of Effect-TS/tsgo `internal/rules/unnecessary_fail_yieldable_error.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

// UnnecessaryFailYieldableError suggests yielding yieldable errors directly
// instead of wrapping with Effect.fail.


// UnnecessaryFailYieldableErrorMatch holds the AST nodes needed by both the
// diagnostic rule and the quick-fix for the unnecessaryFailYieldableError pattern.
#[derive(Clone, Debug)]
pub struct UnnecessaryFailYieldableErrorMatch {
    /// The source file where this match was found
    pub source_file: Node,
    /// The pre-computed error range for this match
    pub location: TextRange,
    /// The yield* expression node
    pub yield_node: Node,
    /// The Effect.fail(...) call expression (fix replaces this)
    pub call_node: Node,
    /// The first argument to Effect.fail (the replacement text)
    pub fail_argument: Node,
}

// AnalyzeUnnecessaryFailYieldableError finds all yield* Effect.fail(...) calls
// where the argument is a yieldable error type that can be yielded directly.
