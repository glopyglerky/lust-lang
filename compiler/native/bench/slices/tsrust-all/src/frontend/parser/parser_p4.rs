use crate::frontend::prelude::*;

// Go: internal/parser/parser.go lines 4184 to 5533 (yield, arrow functions,
// conditional and binary expressions, unary and update expressions, JSX,
// left-hand-side and member expressions, calls, tagged templates).
//
// PORT: Go evaluates call arguments left to right. Rust cannot pass a
// `&mut self` call as an argument to another `&mut self` call, so each such
// argument is moved into a local first, in the Go order.


// Go: parser.go:4503 typeHasArrowFunctionBlockingParseError
// If true, we should abort parsing an error function.

// Go: parser.go:4635 shouldConsumeBinaryOperator
// shouldConsumeBinaryOperator reports whether an operator binds before the operator represented by currentPrecedence.
// At equal precedence, only the right-associative exponentiation operator binds first.
#[must_use]
pub fn should_consume_binary_operator(
    operator: SyntaxKind,
    operator_precedence: OperatorPrecedence,
    current_precedence: OperatorPrecedence,
) -> bool {
    if operator_precedence > current_precedence {
        return true;
    }
    operator_precedence == current_precedence && operator == SyntaxKind::AsteriskAsteriskToken
}
