//! Port of Effect-TS/tsgo `internal/layergraph` (`types.go`, `extract.go`,
//! `outline.go`). It models how Effect Layers are composed in source code.
//! It builds directed graphs where nodes represent Layer expressions and
//! edges represent composition relationships (pipe, call, array literal,
//! symbol).
//!
//! PORT: `format.go`, `magic.go`, `mermaidurl.go` and `providers.go` are not
//! ported: only hover and refactors use them.

use crate::effect::graph::{Direction, Graph, NodeIndex};
use crate::effect::typeparser::*;
use crate::prelude::*;

// ---------------------------------------------------------------------------
// types.go
// ---------------------------------------------------------------------------

/// Go `LayerGraphNodeInfo`: holds data for each node in the layer graph.
#[derive(Clone, Debug, Default)]
pub struct LayerGraphNodeInfo {
    /// The AST expression node
    pub node: Node,
    /// The node used for display (e.g. variable name instead of initializer)
    pub display_node: Node,
    /// Parsed Layer type (ROut, E, RIn) or nil
    pub layer_type: Option<Rc<Layer>>,
    /// Individual service types from ROut (unrolled intersection, Never filtered)
    pub provides: Vec<TypeId>,
    /// Provides minus pass-through (types also in Requires)
    pub actual_provides: Vec<TypeId>,
    /// Individual service types from RIn (unrolled intersection, Never filtered)
    pub requires: Vec<TypeId>,
}

/// Go `EdgeRelationship`: the type of relationship between two nodes in the
/// layer graph.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum EdgeRelationship {
    /// Go `EdgeRelationshipCall`
    #[default]
    Call,
    /// Go `EdgeRelationshipPipe`
    Pipe,
    /// Go `EdgeRelationshipArrayLiteral`
    ArrayLiteral,
    /// Go `EdgeRelationshipSymbol`
    Symbol,
}

impl EdgeRelationship {
    /// The Go string value.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            EdgeRelationship::Call => "call",
            EdgeRelationship::Pipe => "pipe",
            EdgeRelationship::ArrayLiteral => "arrayLiteral",
            EdgeRelationship::Symbol => "symbol",
        }
    }
}

/// Go `LayerGraphEdgeInfo`: describes the relationship between two nodes in
/// the layer graph. It is a discriminated union keyed on Relationship.
// PORT: Go's zero `Relationship` is "", which no Go code creates; the port
// has no empty variant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LayerGraphEdgeInfo {
    /// One of: "call", "pipe", "arrayLiteral", "symbol"
    pub relationship: EdgeRelationship,
    /// Only meaningful for "call" edges (0-based argument position)
    pub argument_index: i32,
    /// Only meaningful for "arrayLiteral" edges (0-based element position)
    pub index: i32,
}

impl LayerGraphEdgeInfo {
    // Go: LayerGraphEdgeInfo.MarshalJSON
    /// Produces JSON matching the reference format, only including fields
    /// relevant to each relationship type:
    ///
    /// ```text
    /// {"relationship":"call","argumentIndex":0}
    /// {"relationship":"pipe"}
    /// {"relationship":"arrayLiteral","index":0}
    /// {"relationship":"symbol"}
    /// ```
    // PORT: the relationship strings need no JSON escaping, so the port
    // writes the text that Go's `json.Marshal` gives.
    #[must_use]
    pub fn marshal_json(&self) -> String {
        match self.relationship {
            EdgeRelationship::Call => format!(
                "{{\"relationship\":\"{}\",\"argumentIndex\":{}}}",
                self.relationship.as_str(),
                self.argument_index
            ),
            EdgeRelationship::ArrayLiteral => format!(
                "{{\"relationship\":\"{}\",\"index\":{}}}",
                self.relationship.as_str(),
                self.index
            ),
            _ => format!("{{\"relationship\":\"{}\"}}", self.relationship.as_str()),
        }
    }
}

/// Go `LayerOutlineGraphNodeInfo`: holds data for nodes in the simplified
/// outline graph.
#[derive(Clone, Debug, Default)]
pub struct LayerOutlineGraphNodeInfo {
    /// The AST node
    pub node: Node,
    /// The node used for display
    pub display_node: Node,
    /// Service types provided
    pub provides: Vec<TypeId>,
    /// Actual (non-pass-through) provides
    pub actual_provides: Vec<TypeId>,
    /// Service types required
    pub requires: Vec<TypeId>,
}

/// Go `LayerMagicNode`: represents a single node in the layer magic result,
/// annotated with flags that determine which Layer.* combinator to use.
#[derive(Clone, Debug, Default)]
pub struct LayerMagicNode {
    /// Whether this node should be merged (provides a target output type)
    pub merges: bool,
    /// Whether this node provides services
    pub provides: bool,
    /// The layer expression AST node
    pub node: Node,
    /// Service types provided
    pub provided_types: Vec<TypeId>,
    /// Actual (non-pass-through) provides
    pub actual_provided_types: Vec<TypeId>,
    /// Service types required
    pub required_types: Vec<TypeId>,
}

/// Go `LayerMagicResult`: holds the output of ConvertOutlineGraphToLayerMagic.
#[derive(Clone, Debug, Default)]
pub struct LayerMagicResult {
    /// Ordered list of layer nodes with merge/provide flags
    pub nodes: Vec<LayerMagicNode>,
    /// Target output types not satisfied by any node
    pub missing_output_types: Vec<TypeId>,
}

/// Go `ProviderRequirerKind`: distinguishes between providers and requirers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProviderRequirerKind {
    /// Go `ProviderRequirerKindProvided`
    Provided,
    /// Go `ProviderRequirerKindRequired`
    Required,
}

impl ProviderRequirerKind {
    /// The Go string value.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            ProviderRequirerKind::Provided => "provided",
            ProviderRequirerKind::Required => "required",
        }
    }
}

/// Go `ProviderRequirerInfo`: summarizes which leaf layers provide or
/// require a service type.
#[derive(Clone, Debug)]
pub struct ProviderRequirerInfo {
    /// "provided" or "required"
    pub kind: ProviderRequirerKind,
    /// The service type
    pub type_: TypeId,
    /// The leaf nodes that provide/require this type
    pub nodes: Vec<Node>,
    /// Display nodes for those leaves
    pub display_nodes: Vec<Node>,
}

/// Go `ExtractLayerGraphOptions`: controls the behavior of ExtractLayerGraph.
#[derive(Clone, Copy, Debug, Default)]
pub struct ExtractLayerGraphOptions {
    /// Treat array literals [l1, l2] as implicit Layer.mergeAll (default: false)
    pub array_literal_as_merge: bool,
    /// Only explode calls to Layer module APIs (default: false)
    pub explode_only_layer_calls: bool,
    /// How many levels deep to follow identifier references (default: 0)
    pub follow_symbols_depth: i32,
    /// Do not decompose pipe/call/array expressions (default: false)
    pub skip_explode: bool,
}

// ---------------------------------------------------------------------------
// extract.go
// ---------------------------------------------------------------------------

/// Go `workItem`: represents a node to visit in the DFS work queue.
#[derive(Clone, Copy, Debug)]
pub struct WorkItem {
    pub node: Node,
    pub depth: i32,
}

// Go: ExtractLayerGraph

// Go: extractNodeInfo

// Go: getDisplayNode

// Go: getAdjustedNode

// Go: isSimpleIdentifier

// Go: isLayerModuleCall

// Go: findLayerModuleName

// Go: collectChildIndices
/// Gathers graph node indices for the given AST nodes, filtering out nodes
/// that aren't in the graph.
fn collect_child_indices(
    nodes: &[Node],
    node_to_graph_index: &FxHashMap<Node, NodeIndex>,
    g: &Graph<LayerGraphNodeInfo, LayerGraphEdgeInfo>,
) -> Vec<NodeIndex> {
    let mut indices = Vec::new();
    for n in nodes {
        if let Some(&idx) = node_to_graph_index.get(n) {
            if g.has_node(idx) {
                indices.push(idx);
            }
        }
    }
    indices
}

// Go: removePartialChildren
/// Removes graph nodes for the given AST nodes.
fn remove_partial_children(
    nodes: &[Node],
    node_to_graph_index: &mut FxHashMap<Node, NodeIndex>,
    g: &mut Graph<LayerGraphNodeInfo, LayerGraphEdgeInfo>,
) {
    for n in nodes {
        if let Some(&idx) = node_to_graph_index.get(n) {
            g.remove_node(idx);
            node_to_graph_index.remove(n);
        }
    }
}

// ---------------------------------------------------------------------------
// outline.go
// ---------------------------------------------------------------------------

// Go: ExtractOutlineGraph
