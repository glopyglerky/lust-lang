//! Port of Go `checker/checker.go` lines 2968-3873 (unit checker-04):
//! type node checks, function declaration checks, overload agreement checks,
//! block and if statement checks, and known-truthy condition checks.

use crate::prelude::*;

// PORT: Go `core.OrElse(a, b)` for node handles.
fn or_else_node_p04(a: Node, b: Node) -> Node {
    if a.is_some() { a } else { b }
}

// PORT: Go `core.OrElse(a, b)` for symbol handles.
fn or_else_symbol_p04(a: SymbolId, b: SymbolId) -> SymbolId {
    if a.is_some() { a } else { b }
}

// PORT: Go `core.ElementOrNil(slice, i)`.
fn element_or_nil_p04(nodes: &[Node], i: usize) -> Node {
    if i < nodes.len() { nodes[i] } else { Node::NIL }
}

