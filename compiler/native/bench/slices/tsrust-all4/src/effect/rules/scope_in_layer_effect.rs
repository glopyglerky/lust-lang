//! Port of Effect-TS/tsgo `internal/rules/scope_in_layer_effect.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



/// ScopeInLayerEffectMatch holds the AST nodes needed by both the diagnostic rule
/// and the quick-fix for the scopeInLayerEffect pattern.
pub struct ScopeInLayerEffectMatch {
    pub source_file: Node,
    /// The pre-computed error range for this match
    pub location: TextRange,
    /// The Layer constructor callee; nil for class declaration matches
    pub callee: Node,
    /// The replaceable property name (e.g., "effect" in Layer.effect); nil for classes, named imports, and local aliases
    pub method_identifier: Node,
}




