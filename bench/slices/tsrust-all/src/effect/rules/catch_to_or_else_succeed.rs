//! Port of Effect-TS/tsgo `internal/rules/catch_to_or_else_succeed.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

// Go: rules.CatchToOrElseSucceed


// Go: rules.CatchToOrElseSucceedMatch
/// CatchToOrElseSucceedMatch holds the AST nodes needed by both the diagnostic rule
/// and the quick-fix for the catchToOrElseSucceed pattern.
#[derive(Clone)]
pub struct CatchToOrElseSucceedMatch {
    pub source_file: Node,
    pub location: TextRange,
    pub callee: Node,
    pub callee_name_node: Node,
    pub catch_method_name: String,
    pub succeed_call_expression: Node,
    pub succeed_argument: Node,
}

// Go: rules.AnalyzeCatchToOrElseSucceed
