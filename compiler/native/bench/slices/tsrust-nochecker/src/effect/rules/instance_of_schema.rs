//! Port of Effect-TS/tsgo `internal/rules/instance_of_schema.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



/// InstanceOfSchemaMatch holds the AST nodes needed by both the diagnostic rule
/// and the quick-fix for the instanceOfSchema pattern.
pub struct InstanceOfSchemaMatch {
    pub source_file: Node,
    /// The pre-computed error range for this match
    pub location: TextRange,
    /// The full BinaryExpression node (the instanceof expression)
    pub instance_of_node: Node,
    /// Left operand of instanceof (the value being tested)
    pub left_expr: Node,
    /// Right operand of instanceof (the schema type reference)
    pub right_expr: Node,
}

