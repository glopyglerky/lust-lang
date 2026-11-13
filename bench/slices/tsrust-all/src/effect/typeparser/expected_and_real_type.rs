//! Port of Effect-TS/tsgo `internal/typeparser/expected_and_real_type.go`.

use crate::effect::typeparser::*;
use crate::prelude::*;
use std::collections::VecDeque;

/// Go `ExpectedAndRealType`: a pair of expected and actual types at an
/// assignment site. This is used by diagnostic rules that need to compare
/// what type was expected at a location versus what type was actually
/// provided.
#[derive(Clone, Copy, Debug)]
pub struct ExpectedAndRealType {
    /// The location node (for diagnostic reporting)
    pub node: Node,
    /// The type expected at this location
    pub expected_type: TypeId,
    /// The actual value node
    pub value_node: Node,
    /// The actual type of the value
    pub real_type: TypeId,
}



