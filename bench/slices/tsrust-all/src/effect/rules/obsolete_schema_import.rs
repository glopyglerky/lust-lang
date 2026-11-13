//! Port of Effect-TS/tsgo `internal/rules/obsolete_schema_import.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

// Go: rules.ObsoleteSchemaImport


// Go: rules.ObsoleteSchemaImportMatch
#[derive(Clone)]
pub struct ObsoleteSchemaImportMatch {
    pub source_file: Node,
    pub location: TextRange,
    pub imported_specifier: String,
}

// Go: rules.AnalyzeObsoleteSchemaImport
// PORT: Go also returns nil for a nil type parser; the Rust one is never nil.
