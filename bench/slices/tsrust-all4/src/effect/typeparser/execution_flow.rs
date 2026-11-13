//! Port of Effect-TS/tsgo `internal/typeparser/execution_flow.go`.

use crate::effect::graph::{Graph, NodeIndex};
use crate::effect::typeparser::*;
use crate::prelude::*;

/// Go `ExecutionNodeKind`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExecutionNodeKind {
    Value,
    Function,
    LogicMerge,
    Transform,
}

impl ExecutionNodeKind {
    /// The Go string value.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            ExecutionNodeKind::Value => "value",
            ExecutionNodeKind::Function => "function",
            ExecutionNodeKind::LogicMerge => "logicMerge",
            ExecutionNodeKind::Transform => "transform",
        }
    }
}

/// Go `ExecutionNode`.
#[derive(Clone)]
pub struct ExecutionNode {
    pub kind: ExecutionNodeKind,
    pub node: Node,
    pub type_: TypeId,

    // Transform nodes preserve the original AST in Node and optionally expose a
    // normalized callee/args view once the visitor reaches that node.
    pub callee: Node,
    pub args: Vec<Node>,
}

/// Go `ExecutionLinkKind`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExecutionLinkKind {
    UsedBy,
    Pipe,
    PotentialReturn,
    Yieldable,
    Parameter,
    TransformArg,
    TransformCallee,
}

impl ExecutionLinkKind {
    /// The Go string value.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            ExecutionLinkKind::UsedBy => "usedBy",
            ExecutionLinkKind::Pipe => "pipe",
            ExecutionLinkKind::PotentialReturn => "potentialReturn",
            ExecutionLinkKind::Yieldable => "yieldable",
            ExecutionLinkKind::Parameter => "parameter",
            ExecutionLinkKind::TransformArg => "transformArg",
            ExecutionLinkKind::TransformCallee => "transformCallee",
        }
    }
}

/// Go `ExecutionLink`.
#[derive(Clone)]
pub struct ExecutionLink {
    pub kind: ExecutionLinkKind,
    pub node: Node,
}

/// Go `ExecutionFlow = graph.Graph[ExecutionNode, ExecutionLink]`.
pub type ExecutionFlow = Graph<ExecutionNode, ExecutionLink>;

/// Go `GraphSlice`.
/// PORT: Go `Leading` and `Trailing` are `*graph.NodeIndex` that are always
/// set (`buildSlice`, or copied from another slice) and only read through
/// `*`, so the port keeps the index values.
#[derive(Clone, Copy, Debug)]
pub struct GraphSlice {
    pub leading: NodeIndex,
    pub trailing: NodeIndex,
}


/// Go pointer equality of two `*GraphSlice`.
fn same_graph_slice(a: &Option<Rc<GraphSlice>>, b: &Option<Rc<GraphSlice>>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => Rc::ptr_eq(a, b),
        _ => false,
    }
}


/// Go `parsedSingleArgInlineCallTransform`.
pub struct ParsedSingleArgInlineCallTransform {
    pub subject: Node,
    pub transform: Node,
}

