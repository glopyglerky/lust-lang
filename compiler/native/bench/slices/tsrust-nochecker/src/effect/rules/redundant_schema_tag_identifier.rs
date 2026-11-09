//! Port of Effect-TS/tsgo `internal/rules/redundant_schema_tag_identifier.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



// RedundantSchemaTagIdentifierMatch holds the AST nodes needed by both the diagnostic rule
// and the quick-fix for the redundantSchemaTagIdentifier pattern.
#[derive(Clone, Debug)]
pub struct RedundantSchemaTagIdentifierMatch {
    pub source_file: Node,
    /// Pre-computed error range for the diagnostic (on KeyStringLiteral)
    pub location: TextRange,
    /// The redundant identifier string literal in the inner call
    pub key_string_literal: Node,
}

// AnalyzeRedundantSchemaTagIdentifier finds all class declarations where the identifier
// string literal equals the tag value in Schema.TaggedClass/TaggedError/TaggedRequest.

