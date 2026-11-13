//! Port of Effect-TS/tsgo `internal/rules/multiple_effect_provide.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



/// MultipleEffectProvideMatch holds the AST nodes needed by both the
/// diagnostic rule and the quick-fix for the multipleEffectProvide pattern.
// Go: rules/multiple_effect_provide.go MultipleEffectProvideMatch
#[derive(Clone, Debug)]
pub struct MultipleEffectProvideMatch {
    /// The source file where the diagnostic should be reported
    pub source_file: Node,
    /// The diagnostic span (error range on the first call in the chunk)
    pub location: TextRange,
    /// The list of call expression nodes in the consecutive provide chain (2+ elements)
    pub chunk: Vec<Node>,
    /// The layer argument nodes from each Effect.provide(layer) call
    pub layer_args: Vec<Node>,
    /// The Effect module identifier node from the first chunk callee
    pub effect_module_node: Node,
}

