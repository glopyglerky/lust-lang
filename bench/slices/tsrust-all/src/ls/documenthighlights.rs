//! Port of Go `ls/documenthighlights.go`.
//!
//! PORT: Go `[]*lsproto.DocumentHighlight` and `[]*ast.Node` results are
//! `Vec`s; a Go nil slice is an empty `Vec` (every caller only tests the
//! length). Go `*ast.SourceFile` is the file root `Node`.

use crate::ls::prelude::*;

use crate::spanmap::Feature;


// Go: ls/documenthighlights.go:112 combineMultiDocumentHighlights
// PORT: Go keeps pointers to the combined documents in `byURI` and appends
// through them; here `by_uri` holds the index into `combined_documents`.
// Go map order is not used for output.
pub fn combine_multi_document_highlights(
    results: Vec<lsproto::MultiDocumentHighlightsOrNull>,
) -> lsproto::MultiDocumentHighlightsOrNull {
    let mut by_uri: FxHashMap<lsproto::DocumentUri, usize> = FxHashMap::default();
    let mut seen: FxHashMap<lsproto::DocumentUri, FxHashSet<lsproto::Range>> = FxHashMap::default();
    let mut combined_documents: Vec<lsproto::MultiDocumentHighlight> = Vec::new();
    for result in results {
        let Some(documents) = result.multi_document_highlights else {
            continue;
        };
        for document in documents {
            let index = match by_uri.get(&document.uri) {
                Some(&index) => index,
                None => {
                    by_uri.insert(document.uri.clone(), combined_documents.len());
                    combined_documents.push(lsproto::MultiDocumentHighlight {
                        uri: document.uri.clone(),
                        ..Default::default()
                    });
                    combined_documents.len() - 1
                }
            };
            let ranges = seen.entry(document.uri.clone()).or_default();
            for highlight in document.highlights {
                if ranges.insert(highlight.range) {
                    combined_documents[index].highlights.push(highlight);
                }
            }
        }
    }
    lsproto::MultiDocumentHighlightsOrNull {
        multi_document_highlights: Some(combined_documents),
    }
}


// Go: ls/documenthighlights.go:340 getIfElseKeywords
// PORT: Go takes `*ast.IfStatement`; here the IfStatement node.

// Go: ls/documenthighlights.go:383 getReturnOccurrences

// Go: ls/documenthighlights.go:412 aggregateOwnedThrowStatements

// Go: ls/documenthighlights.go:441 flatMapChildren

// Go: ls/documenthighlights.go:454 getThrowOccurrences

// Go: ls/documenthighlights.go:489 getThrowStatementOwner
// For lack of a better name, this function takes a throw statement and returns the
// nearest ancestor that is a try-block (whose try statement has a catch clause),
// function-block, or source file.

// Go: ls/documenthighlights.go:512 getTryCatchFinallyOccurrences

// Go: ls/documenthighlights.go:536 getSwitchCaseDefaultOccurrences

// Go: ls/documenthighlights.go:563 aggregateAllBreakAndContinueStatements

// Go: ls/documenthighlights.go:573 ownsBreakOrContinueStatement

// Go: ls/documenthighlights.go:581 getBreakOrContinueOwner

// Go: ls/documenthighlights.go:611 isLabeledBy
// Whether or not a 'node' is preceded by a label of the given string.
// Note: 'node' cannot be a SourceFile.

// Go: ls/documenthighlights.go:623 getBreakOrContinueStatementOccurrences

// Go: ls/documenthighlights.go:635 getLoopBreakContinueOccurrences

// Go: ls/documenthighlights.go:663 getAsyncAndAwaitOccurrences

// Go: ls/documenthighlights.go:692 getYieldOccurrences

// Go: ls/documenthighlights.go:715 traverseWithoutCrossingFunction

// Go: ls/documenthighlights.go:725 getModifierOccurrences

// Go: ls/documenthighlights.go:738 getNodesToSearchForModifier

// Go: ls/documenthighlights.go:789 findModifier
