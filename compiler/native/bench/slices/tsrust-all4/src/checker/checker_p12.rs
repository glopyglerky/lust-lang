//! Port of `checker/checker.go` lines 10269-11171 (createUnionSignature
//! through checkIdentifier).

use crate::prelude::*;

// PORT: Go `getInstantiationExpressionType` keeps its state in closure
// variables shared by nested recursive closures. This struct holds that
// shared state; the closures are the `instantiation_expression_*` methods.
struct InstantiationExpressionTypeState {
    node: Node,
    type_arguments: Vec<Node>,
    has_some_applicable_signature: bool,
    non_applicable_type: TypeId,
}


