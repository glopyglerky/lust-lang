//! Port of Go `ls/utilities.go`.
//!
//! PORT: Go package-private helpers here are called from other files of
//! package `ls`, so they are `pub`. Helpers that read symbol data and have no
//! checker parameter in Go take `symbols: &SymbolArena` first (the arena that
//! holds the symbol, normally `&checker.symbols`), as the ast rule in
//! PORTING.md does. Go `func(*ast.Symbol) *ast.Symbol` callbacks that run
//! while a checker is borrowed take the checker as their first argument.

use crate::ls::prelude::*;

use crate::spanmap::{Feature, Fidelity};
use std::borrow::Cow;
use std::cell::Cell;

// Go: ls/utilities.go:26 quoteReplacer
// PORT: Go `strings.NewReplacer("'", `\'`, `\"`, `"`)`. The pairs are kept in
// Go argument order; `quote_replacer_replace` is `Replace`.
const QUOTE_REPLACER: [(&str, &str); 2] = [("'", "\\'"), ("\\\"", "\"")];


// Go: ls/utilities.go:28 IsInString

// Go: ls/utilities.go:48 isModuleSpecifierLike

// Go: ls/utilities.go:62 getNonModuleSymbolOfMergedModuleSymbol

// Go: ls/utilities.go:73 getLocalSymbolForExportSpecifier
// PORT: Go `*ast.ExportSpecifier` is the `Node`.

// Go: ls/utilities.go:82 isExportSpecifierAlias

// Go: ls/utilities.go:95 isInComment
// PORT: Go `*ast.CommentRange` is `Option<CommentRange>`.

// Go: ls/utilities.go:99 positionBelongsToNode

// Go: ls/utilities.go:103 PossibleTypeArgumentInfo
// PORT: the Go fields are package-private; other `ls` files read them.
#[derive(Clone, Copy, Debug, Default)]
pub struct PossibleTypeArgumentInfo {
    pub called: Node,
    pub n_type_arguments: i32,
}

// Go: ls/utilities.go:109 getPossibleTypeArgumentsInfo
// Get info for an expression like `f <` that may be the start of type arguments.
// PORT: Go returns `*PossibleTypeArgumentInfo`; nil is `None`.

// Go: ls/utilities.go:191 isNameOfModuleDeclaration

// Go: ls/utilities.go:198 isExpressionOfExternalModuleImportEqualsDeclaration

// Go: ls/utilities.go:202 isNamespaceReference

// Go: ls/utilities.go:206 isQualifiedNameNamespaceReference

// Go: ls/utilities.go:220 isPropertyAccessNamespaceReference

// Go: ls/utilities.go:240 isThis

// Go: ls/utilities.go:253 isTypeReference

// Go: ls/utilities.go:277 isInRightSideOfInternalImportEqualsDeclaration


// Go: ls/utilities.go:296 createRangeFromNode


// Go: ls/utilities.go:312 quote
// PORT: Go passes the preferences by value; here by reference.

// Go: ls/utilities.go:322 typeKeywords
// PORT: Go `*collections.Set[ast.Kind]`; membership is all that is read.
pub static TYPE_KEYWORDS: [SyntaxKind; 20] = [
    SyntaxKind::AnyKeyword,
    SyntaxKind::AssertsKeyword,
    SyntaxKind::BigIntKeyword,
    SyntaxKind::BooleanKeyword,
    SyntaxKind::FalseKeyword,
    SyntaxKind::InferKeyword,
    SyntaxKind::KeyOfKeyword,
    SyntaxKind::NeverKeyword,
    SyntaxKind::NullKeyword,
    SyntaxKind::NumberKeyword,
    SyntaxKind::ObjectKeyword,
    SyntaxKind::ReadonlyKeyword,
    SyntaxKind::StringKeyword,
    SyntaxKind::SymbolKeyword,
    SyntaxKind::TypeOfKeyword,
    SyntaxKind::TrueKeyword,
    SyntaxKind::VoidKeyword,
    SyntaxKind::UndefinedKeyword,
    SyntaxKind::UniqueKeyword,
    SyntaxKind::UnknownKeyword,
];

// Go: ls/utilities.go:345 isTypeKeyword
pub fn is_type_keyword(kind: SyntaxKind) -> bool {
    TYPE_KEYWORDS.contains(&kind)
}

// Go: ls/utilities.go:349 isSeparator

// Go: ls/utilities.go:353 isLiteralNameOfPropertyDeclarationOrIndexAccess

// Go: ls/utilities.go:377 isObjectBindingElementWithoutPropertyName

// Go: ls/utilities.go:384 isRightSideOfPropertyAccess

// Go: ls/utilities.go:388 isStaticSymbol

// Go: ls/utilities.go:396 isImplementation

// Go: ls/utilities.go:409 isImplementationExpression

// Go: ls/utilities.go:420 isReadonlyTypeOperator

// Go: ls/utilities.go:424 isJumpStatementTarget

// Go: ls/utilities.go:428 isLabelOfLabeledStatement

// Go: ls/utilities.go:432 findReferenceInPosition
// PORT: Go returns the `*ast.FileReference` element; nil is `None`.
pub fn find_reference_in_position(refs: &[FileReference], pos: i32) -> Option<&FileReference> {
    refs.iter().find(|r| r.range.contains_inclusive(pos))
}

// Go: ls/utilities.go:436 getContainingNodeIfInHeritageClause

// Go: ls/utilities.go:448 getContainerNode

// Go: ls/utilities.go:459 getAdjustedLocation
// PORT: Go `sourceFile` may be nil (`Node::NIL`).

// Go: ls/utilities.go:696 getAdjustedLocationForDeclaration

// Go: ls/utilities.go:720 getAdjustedLocationForImportDeclaration
// PORT: Go `*ast.ImportDeclaration` is the `Node`.

// Go: ls/utilities.go:763 getAdjustedLocationForExportDeclaration
// PORT: Go `*ast.ExportDeclaration` is the `Node`.

// Go: ls/utilities.go:791 symbolFlagsHaveMeaning
pub fn symbol_flags_have_meaning(flags: SymbolFlags, meaning: SemanticMeaning) -> bool {
    if meaning == SemanticMeaning::ALL {
        return true;
    }
    if meaning.intersects(SemanticMeaning::VALUE) {
        return flags.intersects(SymbolFlags::VALUE);
    }
    if meaning.intersects(SemanticMeaning::TYPE) {
        return flags.intersects(SymbolFlags::TYPE);
    }
    if meaning.intersects(SemanticMeaning::NAMESPACE) {
        return flags.intersects(SymbolFlags::NAMESPACE);
    }
    false
}

// Go: ls/utilities.go:807 getMeaningFromLocation

// Go: ls/utilities.go:846 getMeaningFromDeclaration

// Go: ls/utilities.go:881 getIntersectingMeaningFromDeclarations

// Go: ls/utilities.go:922 getAllSuperTypeNodes
// Returns the node in an `extends` or `implements` clause of a class or interface.
// PORT: Go returns `[]*ast.HeritageClauseElement` (ExpressionWithTypeArguments
// or TypeReference nodes, tsgo#4797).

// Go: ls/utilities.go:935 getParentSymbolsOfPropertyAccess

// Go: ls/utilities.go:963 getPropertySymbolsFromBaseTypes
// Find symbol of the given property-name and add the symbol to the given result array
// @param symbol a symbol to start searching for the given propertyName
// @param propertyName a name of property to search for
// @param cb a cache of symbol from previous iterations of calling this function to prevent infinite revisiting of the same symbol.
//
//	The value of previousIterationSymbol is undefined when the function is first called.
//
// PORT: `cb` gets the checker as its first argument, because this function
// holds it (PORTING.md func params). The Go `recur` closure is the inner fn.

// Go: ls/utilities.go:996 getPropertySymbolFromBindingElement

// Go: ls/utilities.go:1003 getPropertySymbolOfObjectBindingPatternWithoutPropertyName

// Go: ls/utilities.go:1011 getTargetLabel

// Go: ls/utilities.go:1022 skipConstraint

// Go: ls/utilities.go:1032 caseClauseTrackerState
// PORT: Go `collections.Set[jsnum.Number]` compares keys with float `==`:
// -0 and +0 are one key and NaN is never found. `NumberKey` makes -0 and +0
// one key; `has_value` returns false for NaN. `jsnum.PseudoBigInt` is not
// `Hash`, so its key is `PseudoBigIntKey` (the struct fields, as Go `==`).
#[derive(Clone, Debug, Default)]
pub struct CaseClauseTrackerState {
    pub existing_strings: FxHashSet<String>,
    pub existing_numbers: FxHashSet<NumberKey>,
    pub existing_big_ints: FxHashSet<PseudoBigIntKey>,
}

// Go: ls/utilities.go:1039 trackerAddValue
// string | jsnum.Number
// PORT: Go `any`; the literal values are `LiteralValue`.
pub type TrackerAddValue = LiteralValue;

// Go: ls/utilities.go:1042 trackerHasValue
// string | jsnum.Number | jsnum.PseudoBigInt
pub type TrackerHasValue = LiteralValue;

// Go: ls/utilities.go:1044 caseClauseTracker
// PORT: the values are passed by reference, as other `LiteralValue`
// parameters in package `ls` are.
pub trait CaseClauseTracker {
    fn add_value(&mut self, value: &TrackerAddValue);
    fn has_value(&self, value: &TrackerHasValue) -> bool;
}

impl CaseClauseTracker for CaseClauseTrackerState {
    // Go: ls/utilities.go:1049 addValue
    fn add_value(&mut self, value: &TrackerAddValue) {
        match value {
            LiteralValue::String(v) => {
                self.existing_strings.insert(v.clone());
            }
            LiteralValue::Number(v) => {
                self.existing_numbers.insert(NumberKey::from(*v));
            }
            // PORT: Go `%T` of the value.
            LiteralValue::PseudoBigInt(_) => {
                crate::core::go_panic("Unsupported type: jsnum.PseudoBigInt".to_string())
            }
            LiteralValue::Bool(_) => crate::core::go_panic("Unsupported type: bool".to_string()),
        }
    }

    // Go: ls/utilities.go:1060 hasValue
    fn has_value(&self, value: &TrackerHasValue) -> bool {
        match value {
            LiteralValue::String(v) => self.existing_strings.contains(v),
            LiteralValue::Number(v) => {
                // PORT: Go float `==` never matches NaN (see the struct).
                !v.is_nan() && self.existing_numbers.contains(&NumberKey::from(*v))
            }
            LiteralValue::PseudoBigInt(v) => {
                self.existing_big_ints.contains(&PseudoBigIntKey::from(v))
            }
            // PORT: Go `%T` of the value.
            LiteralValue::Bool(_) => crate::core::go_panic("Unsupported type: bool".to_string()),
        }
    }
}

// Go: ls/utilities.go:1073 newCaseClauseTracker
// PORT: Go returns the `caseClauseTracker` interface.

// Go: ls/utilities.go:1105 RangeContainsRange
pub fn range_contains_range(r1: TextRange, r2: TextRange) -> bool {
    start_end_contains_range(r1.pos(), r1.end(), r2)
}

// Go: ls/utilities.go:1109 startEndContainsRange
pub fn start_end_contains_range(start: i32, end: i32, text_range: TextRange) -> bool {
    start <= text_range.pos() && end >= text_range.end()
}

// Go: ls/utilities.go:1113 getPossibleGenericSignatures

// Go: ls/utilities.go:1129 removeOptionality

// Go: ls/utilities.go:1138 isNoSubstitutionTemplateLiteral

// Go: ls/utilities.go:1142 isTaggedTemplateExpression

// Go: ls/utilities.go:1146 isInsideTemplateLiteral

// Go: ls/utilities.go:1151 isTemplateHead
// Pseudo-literals

// Go: ls/utilities.go:1155 isTemplateTail

// Go: ls/utilities.go:1159 findPrecedingMatchingToken

// Go: ls/utilities.go:1195 findContainingList
// PORT: Go returns `*ast.NodeList`; nil is `NodeList::NIL`.

// Go: ls/utilities.go:1212 getLeadingCommentRangesOfNode
// PORT: Go returns an `iter.Seq` (nil for JSX text); here the collected
// ranges.

// Go: ls/utilities.go:1220 getChildrenFromNonJSDocNode
// Equivalent to Strada's `node.getChildren()` for non-JSDoc nodes.

// Go: ls/utilities.go:1261 getContainingObjectLiteralElement
// Returns the containing object literal property declaration given a possible name node, e.g. "a" in x = { "a": 1 }

// Go: ls/utilities.go:1269 getContainingObjectLiteralElementWorker

// Go: ls/utilities.go:1287 isObjectLiteralOrJsxElement

// Go: ls/utilities.go:1292 nodeSeenTracker
// Return a function that returns true if the given node has not been seen
pub fn node_seen_tracker() -> Box<dyn FnMut(Node) -> bool> {
    let mut seen: FxHashSet<Node> = FxHashSet::default();
    Box::new(move |node: Node| seen.insert(node))
}

// Go: ls/utilities.go:1300 toContextRange
// FindAllReferences.toContextSpan
// PORT: Go `*core.TextRange` param and result are `Option<TextRange>`. Go
// returns the same pointer when `context` is nil.


// Go: ls/utilities.go:1312 getReferenceAtPosition
// PORT: Go returns `*refInfo`; nil is `None`.

// Go: ls/utilities.go:1362 getContextualTypeFromParent

// Go: ls/utilities.go:1381 getContextualTypeFromParentOrAncestorTypeNode

// Go: ls/utilities.go:1398 getAncestorTypeNode

// Go: ls/utilities.go:1409 isSourceFileWithGlobalExports



#[cfg(test)]
mod go_text_slice_tests {
    use super::go_text_slice;
    use crate::scanner_util::{go_byte_offset, go_string_bytes, go_string_from_bytes};

    // R151 reviewer: a `lo` inside the first marker of a real U+FDD0 kept
    // the second marker, which then read as another U+FDD0. Each slice must
    // be Go's slice of the Go bytes at the Go offsets of its bounds.
    #[test]
    fn go_text_slice_is_go_slice_of_go_bytes() {
        let mut go = b"a\xE2\x82\xACb".to_vec(); // a, U+20AC, b
        go.extend_from_slice("\u{FDD0}".as_bytes()); // a real U+FDD0
        go.extend_from_slice(b"c\xFFd\xED\xA0\x80e"); // invalid byte, WTF-8 surrogate
        go.extend_from_slice("\u{1F600}\u{FDD0}".as_bytes());
        let text = go_string_from_bytes(go.clone());
        assert_eq!(go_string_bytes(&text).as_ref(), go.as_slice());
        for lo in 0..=text.len() {
            for hi in lo..=text.len() {
                let (go_lo, go_hi) = (
                    go_byte_offset(&text, lo as i32) as usize,
                    go_byte_offset(&text, hi as i32) as usize,
                );
                let want = go_string_from_bytes(go[go_lo..go_hi].to_vec());
                assert_eq!(
                    go_text_slice(&text, lo as i32, hi as i32),
                    want,
                    "{text:?}[{lo}:{hi}]"
                );
            }
        }
        // `lo` one byte into a real U+FDD0: Go keeps its last 2 bytes.
        let at = text.find('\u{FDD0}').expect("marker");
        assert_eq!(
            go_text_slice(&text, at as i32 + 1, at as i32 + 7),
            go_string_from_bytes(b"\xB7\x90c".to_vec())
        );
    }
}
