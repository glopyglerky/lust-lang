//! Port of Effect-TS/tsgo `internal/rules/layer_merge_all_with_dependencies.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



/// LayerMergeAllWithDependenciesMatch holds the AST nodes needed by both the
/// diagnostic rule and the quick-fix for the layerMergeAllWithDependencies pattern.
#[derive(Clone, Debug)]
pub struct LayerMergeAllWithDependenciesMatch {
    pub source_file: Node,
    /// The error range on the provider argument
    pub location: TextRange,
    /// The full Layer.mergeAll(...) call expression node
    pub call_node: Node,
    /// The specific argument node that is the dependency provider
    pub provider_arg: Node,
    /// Index of the provider argument in the call's argument list
    pub provider_index: i32,
    /// All arguments of the mergeAll call
    pub all_args: Vec<Node>,
    /// Formatted string of provided type names
    pub provided_types: String,
}



