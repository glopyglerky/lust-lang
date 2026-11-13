//! Port of Go `api/encoder/encoder_generated.go` (generated in Go by
//! `_scripts/generate-encoder.ts`). One arm per kind, in Go order.
//!
//! PORT: Go `n := node.AsX()` type assertions are the per-field `Node`
//! accessors, which panic on other kinds like the Go assertion. Go
//! `n.Field != nil` is `node.field().is_some()`; a Go `*NodeList` field uses
//! the `NodeList` accessor (`statement_list`, `parameter_list`, ...).

use crate::api::encoder::prelude::*;

// Go: api/encoder/encoder_generated.go:11 getNodeDataType

/// `get_node_data_type` of a node of kind `kind`.
// PERF: (apiperf2) the encoder reads the kind of a node once.
#[inline]
pub fn node_data_type_of_kind(kind: SyntaxKind) -> u32 {
    match kind {
        SyntaxKind::Identifier
        | SyntaxKind::PrivateIdentifier
        | SyntaxKind::JsxText
        | SyntaxKind::JsDocText
        | SyntaxKind::JsDocLink
        | SyntaxKind::JsDocLinkPlain
        | SyntaxKind::JsDocLinkCode => NODE_DATA_TYPE_STRING,
        SyntaxKind::StringLiteral
        | SyntaxKind::NumericLiteral
        | SyntaxKind::BigIntLiteral
        | SyntaxKind::RegularExpressionLiteral
        | SyntaxKind::NoSubstitutionTemplateLiteral
        | SyntaxKind::TemplateHead
        | SyntaxKind::TemplateMiddle
        | SyntaxKind::TemplateTail
        | SyntaxKind::SourceFile => NODE_DATA_TYPE_EXTENDED_DATA,
        _ => NODE_DATA_TYPE_CHILDREN,
    }
}

// Go: api/encoder/encoder_generated.go:36 getChildrenPropertyMask

// Go: api/encoder/encoder_generated.go:541 getNodeCommonData


// Go: api/encoder/encoder_generated.go:661 recordNodeStrings

// Go: api/encoder/encoder_generated.go:682 recordExtendedData
// PORT: `given` goes to the SourceFile arm (see `get_node_data`).
