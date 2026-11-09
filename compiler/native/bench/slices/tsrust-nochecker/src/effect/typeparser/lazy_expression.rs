//! Port of Effect-TS/tsgo `internal/typeparser/lazy_expression.go`.

use crate::effect::typeparser::*;
use crate::flags_macros::go_flags;
use crate::prelude::*;

/// ParsedLazyExpression represents a parsed arrow function or function expression
/// with its inner expression extracted.
// Go: typeparser/lazy_expression.go ParsedLazyExpression
#[derive(Clone)]
pub struct ParsedLazyExpression {
    /// The original ArrowFunction or FunctionExpression node
    pub node: Node,
    /// Parameter declarations (empty when parsed with thunk=true)
    pub params: Vec<Node>,
    /// The function body as written (Expression or Block)
    pub body: Node,
    /// The inner expression (return value)
    pub expression: Node,
}

// LazyExpressionFlags controls which function shapes ParseLazyExpression accepts.
// PORT: Go assigns `1 << iota` from the second const, so the values are
// 0, 2, 4 and 8, kept here.
// Go: typeparser/lazy_expression.go LazyExpressionFlags

