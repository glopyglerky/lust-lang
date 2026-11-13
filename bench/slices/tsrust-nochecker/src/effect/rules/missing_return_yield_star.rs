//! Port of Effect-TS/tsgo `internal/rules/missing_return_yield_star.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

// Go: rules.MissingReturnYieldStar


// Go: rules.MissingReturnYieldStarMatch
/// MissingReturnYieldStarMatch holds the AST nodes needed by both the diagnostic rule
/// and the quick-fix for the missingReturnYieldStar pattern.
#[derive(Clone)]
pub struct MissingReturnYieldStarMatch {
    /// The source file where the diagnostic should be reported
    pub source_file: Node,
    /// The pre-computed error range for this match
    pub location: TextRange,
    /// The yield* expression node (for diagnostic location)
    pub yield_node: Node,
    /// The expression statement node (for quickfix replacement)
    pub expr_stmt_node: Node,
}

// Go: rules.AnalyzeMissingReturnYieldStar


// Go: rules.shouldReportMissingReturnYieldStar
// PORT: Go also returns false for a nil checker; the type parser always has one.
