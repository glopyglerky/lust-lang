//! Port of Effect-TS/tsgo `internal/rules/overridden_schema_constructor.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

// Go: rules/overridden_schema_constructor.go OverriddenSchemaConstructor


/// OverriddenSchemaConstructorMatch holds the AST nodes needed by both the diagnostic rule
/// and the quick-fixes for the overriddenSchemaConstructor pattern.
// Go: rules/overridden_schema_constructor.go OverriddenSchemaConstructorMatch
#[derive(Clone, Copy)]
pub struct OverriddenSchemaConstructorMatch {
    pub source_file: Node,
    /// The error range for the constructor node
    pub location: TextRange,
    /// The constructor declaration AST node
    pub constructor_node: Node,
    /// Whether the constructor has a body (the static fix is only available when true)
    pub has_body: bool,
}


// Go: rules/overridden_schema_constructor.go checkOverriddenSchemaConstructor

