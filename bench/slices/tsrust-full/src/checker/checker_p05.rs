//! Port of Go `checker/checker.go` lines 3874-4793 (checker-05):
//! statement checks (do/while/for/for-in/for-of/return/with/switch/labeled/
//! throw/try/catch) and class declaration checks (heritage, overrides,
//! index constraints).

use crate::prelude::*;

// PORT: Go `core.OrElse(node, fallback)` for node handles.
fn or_else_node(node: Node, fallback: Node) -> Node {
    if node.is_some() { node } else { fallback }
}

// PORT: Go `core.OrElse(t, fallback)` for type handles.

// PORT: the Go closure `visit` inside isSymbolUsedInBinaryExpressionChain.

// PORT: the Go closure `visit` inside isSymbolUsedInConditionBody.

