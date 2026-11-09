//! Port of Effect-TS/tsgo `internal/rules/missing_effect_error.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



/// MissingEffectErrorMatch holds the analysis results needed by both the
/// diagnostic rule and the quick-fixes for the missingEffectError pattern.
// Go: rules/missing_effect_error.go MissingEffectErrorMatch
#[derive(Clone, Debug)]
pub struct MissingEffectErrorMatch {
    /// The source file where the diagnostic should be reported
    pub source_file: Node,
    /// The diagnostic error range
    pub location: TextRange,
    /// The AST node where the type error occurs
    pub error_node: Node,
    /// The individual error types not handled by the target
    pub unhandled_errors: Vec<TypeId>,
    /// The target Effect's error type (.E)
    pub expected_error_type: TypeId,
    /// The formatted union string of unhandled errors
    pub error_type_str: String,
}



