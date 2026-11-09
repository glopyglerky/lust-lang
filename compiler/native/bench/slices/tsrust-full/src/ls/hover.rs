use crate::ls::prelude::*;

use crate::spanmap::{Feature, Fidelity};

// Go: ls/hover.go:20 symbolFormatFlags

// Go: ls/hover.go:24 typeFormatFlags


// Go: ls/hover.go:204 documentationLocationMapper
/// `l.documentationLocationMapper(feature)` or `noMappedLocation`.
// PORT: Go passes the func value; here a reference to the closure.
pub type DocumentationLocationMapper<'a> =
    &'a dyn Fn(Node, TextRange) -> (lsproto::Location, Fidelity);


// Go: ls/hover.go:212 getDocumentationForSymbol
// getDocumentationForSymbol tries each documentation source in turn (call-signature documentation,
// declaration JSDoc, root-symbol JSDoc, alias target JSDoc) and returns the first non-empty result,
// formatted for contentFormat. commentOnly restricts the result to the JSDoc summary, excluding the
// @tag section.

// Go: ls/hover.go:231 documentationFromSignature

// Go: ls/hover.go:249 documentationFromAlias

// Go: ls/hover.go:278 documentationFromRootSymbols

// Go: ls/hover.go:306 getDocumentationFromDeclaration

// Go: ls/hover.go:396 formatQuickInfo
pub fn format_quick_info(quick_info: &str) -> String {
    let mut b = String::with_capacity(32);
    write_code(&mut b, "typescript", quick_info);
    b
}

// Go: ls/hover.go:403 shouldGetType

// Go: ls/hover.go:419 symbolDisplayInfo

// Go: ls/hover.go:440 classifiedNodeBuilderFlags (local const of getQuickInfoAndDeclarationAtLocation)
// nodeBuilderFlags for classified output (same as signatureHelpNodeBuilderFlags)

// PORT: Go getQuickInfoAndDeclarationAtLocation builds its output with local
// closures (writeTypeClassified, writeSignatureClassified,
// writeSymbolClassified, setDeclaration, writeNewLine, writeSignatures,
// writeTypeParams, canExpandSymbol, tryExpandSymbol, writeSymbol) that share
// the checker, the writer, `vc` and the alias state. Rust closures cannot call
// each other while all of them borrow the checker mutably, so the shared
// state is this struct and each Go closure is a method with the Go name.


// Go: ls/hover.go:426 getQuickInfoAndDeclarationAtLocation
// getQuickInfoAndDeclarationAtLocation builds classified display parts using displayPartsWriter when vsCapability is true.
// When vsCapability is false, it still builds the plain text string but skips classification runs.

// Go: ls/hover.go:951 typeParameterToString
// typeParameterToString renders a type parameter declaration (e.g., "T extends FooType").

// Go: ls/hover.go:955 getNodeForQuickInfo

// Go: ls/hover.go:974 getSymbolAtLocationForQuickInfo

// Go: ls/hover.go:985 getSignaturesAtLocation

// Go: ls/hover.go:996 getCallOrNewExpression

// Go: ls/hover.go:1009 containsTypedefTag

// Go: ls/hover.go:1022 writeCode
pub fn write_code(b: &mut String, lang: &str, code: &str) {
    if code.is_empty() {
        return;
    }
    let mut ticks = 3;
    while code.contains(&"`".repeat(ticks)) {
        ticks += 1;
    }
    for _ in 0..ticks {
        b.push('`');
    }
    b.push_str(lang);
    b.push('\n');
    b.push_str(code);
    b.push('\n');
    for _ in 0..ticks {
        b.push('`');
    }
    b.push('\n');
}

// Go: ls/hover.go:1043 writeComments

// Go: ls/hover.go:1056 writeJSDocLink

// Go: ls/hover.go:1088 writeNameLink

// Go: ls/hover.go:1111 trimCommentPrefix

// Go: ls/hover.go:1115 writeMarkdownLink

// Go: ls/hover.go:1123 writeOptionalEntityName

// Go: ls/hover.go:1130 writeQuotedString

// Go: ls/hover.go:1140 getEntityNameString

// Go: ls/hover.go:1146 writeEntityNameParts
