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

// Go: removePartialChildren

// ---------------------------------------------------------------------------
// outline.go
// ---------------------------------------------------------------------------

// Go: ExtractOutlineGraph
