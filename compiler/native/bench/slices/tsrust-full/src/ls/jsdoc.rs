use crate::ls::prelude::*;

// Port of Go `internal/ls/jsdoc.go` (tsgo#4424): the symbol documentation
// comment and JSDoc tags that the API returns, and the JSDoc lookup helpers
// that tsgo#4893 moved here from `hover.go`.

// Go: ls/jsdoc.go:18 JSDocTagInfo
/// JSDocTagInfo mirrors Strada's `JSDocTagInfo`, but renders the tag's text as a
/// plain string instead of `SymbolDisplayPart[]`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct JSDocTagInfo {
    pub name: String,
    pub text: String,
}

// Go: ls/jsdoc.go:28 GetSymbolDocumentationComment

// Go: ls/jsdoc.go:51 GetSymbolJSDocTags

// Go: ls/jsdoc.go:85 declarationJSDocTags

// Go: ls/jsdoc.go:103 getJSDocTagText

// Go: ls/jsdoc.go:164 getJSDoc

// Go: ls/jsdoc.go:168 getJSDocOrTag

// Go: ls/jsdoc.go:239 getMatchingJSDocTag

// Go: ls/jsdoc.go:254 getJSDocParameterTagByPosition
// getJSDocParameterTagByPosition finds a JSDoc @param tag for a binding pattern parameter by position.
// Since binding patterns don't have a simple name, we match the @param tag at the same index as the parameter.

// Go: ls/jsdoc.go:297 isMatchingParameterTag

// Go: ls/jsdoc.go:301 isMatchingTemplateTag

// Go: ls/jsdoc.go:305 isNodeWithName

// Go: ls/jsdoc.go:310 noMappedLocation
pub fn no_mapped_location(_: Node, _: TextRange) -> (lsproto::Location, crate::spanmap::Fidelity) {
    (lsproto::Location::default(), crate::spanmap::Fidelity::NONE)
}
