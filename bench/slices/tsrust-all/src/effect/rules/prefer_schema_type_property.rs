//! Port of Effect-TS/tsgo `internal/rules/prefer_schema_type_property.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



pub struct PreferSchemaTypePropertyMatch {
    pub source_file: Node,
    pub location: TextRange,
    pub type_reference: Node,
    pub schema_name: Node,
    pub schema_name_text: String,
}

// Go: rules/prefer_schema_type_property.go AnalyzePreferSchemaTypeProperty

// Go: rules/prefer_schema_type_property.go analyzePreferSchemaTypePropertyNode
