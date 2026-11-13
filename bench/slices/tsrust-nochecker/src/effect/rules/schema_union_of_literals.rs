//! Port of Effect-TS/tsgo `internal/rules/schema_union_of_literals.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



/// SchemaUnionOfLiteralsMatch holds the AST nodes needed by both the diagnostic rule
/// and the quick-fix for the schemaUnionOfLiterals pattern.
#[derive(Default)]
pub struct SchemaUnionOfLiteralsMatch {
    pub source_file: Node,
    /// Pre-computed error range for the diagnostic
    pub location: TextRange,
    /// The full Schema.Union(...) call expression to be replaced
    pub union_call_node: Node,
    /// The callee expression (Schema.Literal) from the first argument
    pub first_literal_expression: Node,
    /// All arguments collected from every Schema.Literal(...) call, in order
    pub all_literal_args: Vec<Node>,
}


