//! Port of Effect-TS/tsgo `internal/rules/schema_struct_with_tag.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



/// SchemaStructWithTagMatch holds the AST nodes needed by both the diagnostic rule
/// and the quick-fix for the schemaStructWithTag pattern.
// Go: rules/schema_struct_with_tag.go SchemaStructWithTagMatch
#[derive(Clone, Debug, Default)]
pub struct SchemaStructWithTagMatch {
    pub source_file: Node,
    /// Pre-computed error range for the diagnostic
    pub location: TextRange,
    /// The full Schema.Struct(...) call expression
    pub call_node: Node,
    /// The string value extracted from Schema.Literal("...")
    pub tag_value: String,
    /// All property assignment nodes in the object literal except _tag
    pub other_properties: Vec<Node>,
    /// The expression node for the Schema part of Schema.Struct
    pub schema_expr: Node,
}


