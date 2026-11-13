//! Port of Effect-TS/tsgo `internal/rules/new_schema_class.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



pub struct NewSchemaClassMatch {
    pub source_file: Node,
    pub location: TextRange,
    pub new_node: Node,
    pub class_expr: Node,
    pub class_text: String,
    pub arguments: Vec<Node>,
}

// Go: rules/new_schema_class.go AnalyzeNewSchemaClass

// Go: rules/new_schema_class.go isV4SchemaClassExpression
