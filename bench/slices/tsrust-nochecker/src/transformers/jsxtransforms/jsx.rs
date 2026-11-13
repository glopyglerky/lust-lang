//! Port of Go `transformers/jsxtransforms/jsx.go`: the JSX transform for
//! `jsx: react`, `react-jsx` and `react-jsxdev`.
//!
//! PORT: Go `tx.Visitor().Visit(n)` is the raw visit callback, so it is
//! `self.visit(n)` here. The other visitor methods (`VisitEachChild`,
//! `VisitSlice`) go through `TransformerVisit::with_visitor`.
//!
//! PORT: the entity table (Go `var entities`) is in `entities.rs`.

use super::entities::entity_code_point;
use crate::prelude::*;
use crate::printer::{AutoGenerateOptions, GeneratedIdentifierFlags};
use crate::transformers::transformer::{
    
};


/// Which Go `tagTransform` method value `visitJsxElement` and
/// `visitJsxSelfClosingElement` picked.
// PORT: Go stores a method value in a local; a bool picks the method here.
#[derive(Clone, Copy)]
enum TagTransform {
    Jsx,
    CreateElement,
}

// Go: transformers/jsxtransforms/jsx.go:32 NewJSXTransformer
// PORT: Go never returns nil here, so this returns a plain `TransformerBox`.





// Go: transformers/jsxtransforms/jsx.go:141 hasKeyAfterPropsSpread

// Go: transformers/jsxtransforms/jsx.go:161 insertStatementAfterPrologue
// PORT: Go passes a method expression plus its receiver; a closure does both here.
fn insert_statement_after_prologue(
    mut to: Vec<Node>,
    statement: Node,
    is_prologue_directive: impl Fn(Node) -> bool,
) -> Vec<Node> {
    if statement.is_nil() {
        return to;
    }
    let mut statement_idx = 0;
    // skip all prologue directives to insert at the correct position
    while statement_idx < to.len() {
        if !is_prologue_directive(to[statement_idx]) {
            break;
        }
        statement_idx += 1;
    }
    to.insert(statement_idx, statement);
    to
}

// Go: transformers/jsxtransforms/jsx.go:183 sortImportSpecifiers

// Go: transformers/jsxtransforms/jsx.go:191 getSortedSpecifiers

// Go: transformers/jsxtransforms/jsx.go:481 hasProto

// Go: transformers/jsxtransforms/jsx.go:785 addLineOfJsxText

// Go: transformers/jsxtransforms/jsx.go:810 fixupWhitespaceAndDecodeEntities

// Go: transformers/jsxtransforms/jsx.go:864 decodeEntities

// Go: transformers/jsxtransforms/jsx.go:914 decodeEntity
