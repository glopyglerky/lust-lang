//! Port of `transformers/declarations/transform.go` lines 1 to 752: the
//! transformer state, its constructor, `GetDiagnostics`, the `@internal`
//! checks, the root `visit` dispatch, `transformSourceFile` with its helpers,
//! `setupDiagnosticContext`, `visitDeclarationSubtree`, `checkName`,
//! `transformMappedTypeNode` and `transformHeritageClause`.
//!
//! PORT: Go `tx.Visitor()` is the root visitor. Its `Visit` field is
//! `tx.visit`, so Go `tx.Visitor().Visit(n)` is `self.visit(n)` here. The
//! other root visitor methods go through `with_visitor` (transform_p2.rs).
//! The other Go visitor fields (`cjsExportAssignmentVisitor`,
//! `expressionVisitor`, ...) are not fields; see `transform_source_file`.

use super::diagnostics::{
    
    
    
};
use super::tracker::{};
use super::util::{
    can_produce_diagnostics_kind, 
    
};
use crate::prelude::*;
use crate::printer::{
    CommentRange, SymbolAccessibilityResult, 
};

// Go: transformers/declarations/transform.go:23 ReferencedFilePair
#[derive(Clone)]
pub struct ReferencedFilePair {
    pub file: Node,
    pub r#ref: FileReference,
}

// Go: transformers/declarations/transform.go:46 thisPropertyAssignmentKey
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct ThisPropertyAssignmentKey {
    pub(crate) name: String,
    pub(crate) node: Node,
    pub(crate) is_static: bool,
    pub(crate) is_private: bool,
}

// Go: transformers/declarations/transform.go:53 getThisPropertyAssignmentKey

// Go: transformers/declarations/transform.go:63 DeclarationTransformer
// PORT: Go embeds `transformers.Transformer` (emit context, factory and root
// visitor). The emit context is the `emit_context` field; the factory is
// `emit_context.factory()`; the root visitor is built by `with_visitor`
// (transform_p2.rs). Go also builds six `*ast.NodeVisitor` fields in the
// constructor (bindingNameVisitor, expressionVisitor,
// cjsExportAssignmentVisitor, exportStrippingVisitor, thisPropertyVisitor,
// declareStrippingVisitor). They are not fields: each visitor holds its
// callback's receiver, so the methods that use them build them on demand.
// Go maps keyed by `ast.NodeId` are keyed by `Node`.

// Go: transformers/declarations/transform.go:103 NewDeclarationTransformer
// TODO: Convert to transformers.TransformerFactory signature to allow more automatic composition with other transforms
// PORT: a nil Go `context` is `None` (Go `NewTransformer` then makes a new
// emit context). Go stores `reportExpandoFunctionErrors` as a closure on the
// state; it is `SymbolTrackerSharedState::report_expando_function_errors`.
// The Go visitor fields are built on demand (see the struct comment).






// Go: transformers/declarations/transform.go:210 hasInternalAnnotation

// Go: transformers/declarations/transform.go:276 throwDiagnostic

// Go: transformers/declarations/transform.go:316 nodeOrSyntaxListChildren

// Go: transformers/declarations/transform.go:323 flattenSyntaxLists

// Go: transformers/declarations/transform.go:382 createEmptyExports

// Go: transformers/declarations/transform.go:519 getLibReferences loop body
// PORT: `getLibReferences` and `getTypeReferences` have the same loop over a
// different field; it is shared here.
fn retained_references(refs: &[FileReference]) -> Vec<FileReference> {
    let mut result = Vec::new();
    for r#ref in refs {
        if !r#ref.preserve {
            continue;
        }
        result.push(FileReference {
            range: TextRange::new(-1, -1),
            file_name: r#ref.file_name.clone(),
            resolution_mode: r#ref.resolution_mode,
            preserve: r#ref.preserve,
        });
    }
    result
}

// Go: tspath/path.go:829 GetRelativePathToDirectoryOrUrl
// PORT: tspath has no Go-shaped port outside the private checker copy, so
// this uses `tspath_p17` from checker_p17.rs.

// Go: scanner/scanner.go:2799 GetLeadingCommentRanges
// PORT: Go takes the node factory only to allocate the ranges; it is dropped.
// The Go iterator is collected into a Vec; every caller reads all of it.

// Go: scanner/scanner.go:2803 GetTrailingCommentRanges
// PORT: see get_leading_comment_ranges.


// Go: scanner/scanner.go:2474 isShebangTrivia
// PORT: the scanner_util copy is private.
fn is_shebang_trivia(text: &str, pos: usize) -> bool {
    let bytes = text.as_bytes();
    if bytes.len() < 2 {
        return false;
    }
    assert!(
        pos == 0,
        "Shebangs check must only be done at the start of the file"
    );
    bytes[0] == b'#' && bytes[1] == b'!'
}

// Go: scanner/scanner.go:2484 scanShebangTrivia
// PORT: the scanner_util copy is private.

// Go: scanner/scanner.go:2813 iterateCommentRanges
/*
Returns an iterator over each comment range following the provided position.

Single-line comment ranges include the leading double-slash characters but not the ending
line break. Multi-line comment ranges include the leading slash-asterisk and trailing
asterisk-slash characters.
*/
// PORT: Go yields lazily; this collects the ranges into a Vec.
