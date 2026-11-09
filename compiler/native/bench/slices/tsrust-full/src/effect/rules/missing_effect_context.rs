//! Port of Effect-TS/tsgo `internal/rules/missing_effect_context.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::layergraph;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;


// Go: MissingEffectContext.Run

// Go: findUnhandledContexts

// Go: formatContextTypes

// Go: missingEffectContextRelatedInformation
// PORT: Go also takes `c`; the port uses `ctx.tp.checker`.

/// Go `provideLocation`.
#[derive(Clone, Copy, Debug, Default)]
pub struct ProvideLocation {
    pub layer_node: Node,
    pub location: TextRange,
}

// Go: findRelatedProvideLocation
// PORT: Go returns the zero value when `c` is nil; the port's checker is
// never nil.

// Go: findRelatedLayerProviderDiagnostics
// PORT: Go also takes `c`, and returns nil when `c` or `ctx` is nil; the
// port uses `ctx.tp.checker`, and neither can be nil.

// Go: rootLayerProvidesTypes

// Go: rootProvidesType

// Go: outlineNodeProvidesType

// Go: firstNode
fn first_node(nodes: &[Node]) -> Node {
    if nodes.is_empty() {
        return Node::NIL;
    }
    nodes[0]
}

// Go: findTransformationIndexContainingNode
// PORT: Go returns -1 for a nil flow; the port's flow is never nil.

// Go: nodeContains
