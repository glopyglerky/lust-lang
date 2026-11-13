//! Port of Effect-TS/tsgo `internal/rules/class_self_mismatch.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

// ClassSelfMismatch ensures the Self type parameter matches the class name in
// Effect.Service, Context.Service, Context.Tag, Effect.Tag, Schema.Class,
// Schema.TaggedClass, Schema.TaggedError, Schema.TaggedRequest,
// Schema.RequestClass, and Model.Class declarations.


// ClassSelfMismatchMatch holds the AST nodes needed by both the diagnostic rule
// and the quick-fix for the classSelfMismatch pattern.
#[derive(Clone, Debug)]
pub struct ClassSelfMismatchMatch {
    /// The source file of the match
    pub source_file: Node,
    /// The pre-computed error range for the selfTypeNode
    pub location: TextRange,
    /// The Self type argument node
    pub self_type_node: Node,
    /// The class name identifier
    pub class_name: Node,
    /// The expected name (from the class declaration)
    pub expected_name: String,
    /// The actual name found in the Self type parameter
    pub actual_name: String,
}

// AnalyzeClassSelfMismatch finds all class declarations where the Self type
// parameter does not match the class name.


// extractSelfTypeName extracts the name text from a Self type node.
// For TypeReferenceNode with Identifier typeName → identifier text.
// For TypeReferenceNode with QualifiedName typeName → right identifier text.
// Fallback → source text substring between node pos and end.
