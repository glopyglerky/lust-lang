//! Port of Effect-TS/tsgo `internal/rules/prefer_typed_schema_decoder.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

// PORT: Go keeps a `map[string]string` and ranges over it in random order.
// A callee references at most one of these exports, and the match returns
// on the first hit, so the order does not change the output.
// Go: rules/prefer_typed_schema_decoder.go typedSchemaDecoders
static TYPED_SCHEMA_DECODERS: &[(&str, &str)] = &[
    ("decodeUnknownEffect", "decodeEffect"),
    ("decodeUnknownSync", "decodeSync"),
    ("decodeUnknownExit", "decodeExit"),
    ("decodeUnknownOption", "decodeOption"),
    ("decodeUnknownResult", "decodeResult"),
    ("decodeUnknownPromise", "decodePromise"),
];



// Go: rules/prefer_typed_schema_decoder.go PreferTypedSchemaDecoderMatch
#[derive(Clone, Debug)]
pub struct PreferTypedSchemaDecoderMatch {
    pub source_file: Node,
    pub location: TextRange,
    pub decoder_name: Node,
    pub unknown_name: String,
    pub typed_name: String,
}


// Go: rules/prefer_typed_schema_decoder.go schemaDecoderTransformation

// Go: rules/prefer_typed_schema_decoder.go analyzeTypedSchemaDecoderApplication

// Go: rules/prefer_typed_schema_decoder.go containsUnresolvedTypeVariable

// Go: rules/prefer_typed_schema_decoder.go matchUnknownSchemaDecoder

// Go: rules/prefer_typed_schema_decoder.go schemaDecoderNameNode
