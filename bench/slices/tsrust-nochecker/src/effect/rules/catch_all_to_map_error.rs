//! Port of Effect-TS/tsgo `internal/rules/catch_all_to_map_error.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



/// CatchAllToMapErrorMatch holds the AST nodes needed by both the diagnostic rule
/// and the quick-fix for the catchAllToMapError pattern.
// Go: rules/catch_all_to_map_error.go CatchAllToMapErrorMatch
#[derive(Clone, Debug)]
pub struct CatchAllToMapErrorMatch {
    /// The source file of the match
    pub source_file: Node,
    /// The pre-computed error range for this match
    pub location: TextRange,
    /// The Effect.catch callee node (for diagnostic location)
    pub callee: Node,
    /// The "catch" name node within the PropertyAccessExpression (for text replacement)
    pub callee_name_node: Node,
    /// The catch variant name (e.g. "catch" or "catchAll")
    pub catch_method_name: String,
    /// The Effect.fail(arg) call expression node (for replacement range)
    pub fail_call_expression: Node,
    /// The first argument to Effect.fail (the replacement text)
    pub fail_argument: Node,
}

