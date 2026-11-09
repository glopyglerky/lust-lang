// Go: internal/rules/schema_number.go

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



#[derive(Clone, Debug)]
pub struct SchemaNumberMatch {
    pub source_file: Node,
    pub location: TextRange,
    pub reference_node: Node,
    pub replacement: String,
    pub replacement_identifier: String,
}

// Go: rules/schema_number.go AnalyzeSchemaNumber

// Go: rules/schema_number.go analyzeSchemaNumberReference

// Go: rules/schema_number.go schemaNumberHasFiniteMethodCheck

// Go: rules/schema_number.go schemaNumberHasFinitePipingCheck

// Go: rules/schema_number.go schemaNumberHasFinitePredicate

// Go: rules/schema_number.go schemaNumberNodeContains

/// Go `schemaNumberApi`.
pub struct SchemaNumberApi {
    pub name: &'static str,
    pub replacement: &'static str,
    pub replacement_identifier: &'static str,
}

static SCHEMA_NUMBER_APIS: &[SchemaNumberApi] = &[
    SchemaNumberApi {
        name: "Number",
        replacement: "Schema.Finite",
        replacement_identifier: "Finite",
    },
    SchemaNumberApi {
        name: "NumberFromString",
        replacement: "Schema.FiniteFromString",
        replacement_identifier: "FiniteFromString",
    },
];

// Go: rules/schema_number.go schemaNumberReferenceLocation
