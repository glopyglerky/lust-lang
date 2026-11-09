//! Port of Go `parser/parser.go` lines 5534 to 6822: templates, primary
//! expressions, literals, identifiers, node finishing, lookahead helpers,
//! context flags, pragmas and the JavaScript syntax checks.
//!
//! Unit U8 of the frontend plan (`/tmp/port/frontend/plan.md`). The `Parser`
//! struct and its fields are in `parser_p1.rs` (U4).

use crate::frontend::prelude::*;



// Go: parser/parser.go:6447 isReservedWord
pub fn is_reserved_word(token: SyntaxKind) -> bool {
    (SyntaxKind::FIRST_RESERVED_WORD as u16) <= (token as u16)
        && (token as u16) <= (SyntaxKind::LAST_RESERVED_WORD as u16)
}

// Go: parser/parser.go:6451 attachFileToDiagnostics
// PORT: Go sets the file on shared diagnostic pointers and returns the same
// slice. Rust diagnostics are owned, so the vector is taken and returned.

// Go: parser/parser.go:6461 getCommentPragmas


// Go: parser/parser.go:6469 extractPragmas


// Go: parser/parser.go:6573 match

// Go: parser/parser.go:6577 skipBlanks
fn skip_blanks(text: &str, mut pos: i32) -> i32 {
    let bytes = text.as_bytes();
    while (pos as usize) < bytes.len()
        && (bytes[pos as usize] == b' ' || bytes[pos as usize] == b'\t')
    {
        pos += 1;
    }
    pos
}

// Go: parser/parser.go:6584 skipNonBlanks
fn skip_non_blanks(text: &str, mut pos: i32) -> i32 {
    let bytes = text.as_bytes();
    while (pos as usize) < bytes.len()
        && !matches!(bytes[pos as usize], b' ' | b'\t' | b'\r' | b'\n')
    {
        pos += 1;
    }
    pos
}

// Go: parser/parser.go:6591 skipTo

// Go: parser/parser.go:6602 lineEndPos

// Go: parser/parser.go:6613 extractName

// Go: parser/parser.go:6621 extractQuotedString
// PORT: Go `(string, bool)` is `Option<&str>`.
fn extract_quoted_string(text: &str, mut pos: i32) -> Option<&str> {
    let bytes = text.as_bytes();
    if pos as usize == bytes.len() {
        return None;
    }
    let quote = bytes[pos as usize];
    if quote != b'\'' && quote != b'"' {
        return None;
    }
    pos += 1;
    let start = pos;
    while (pos as usize) < bytes.len() && bytes[pos as usize] != quote {
        pos += 1;
    }
    if pos as usize == bytes.len() {
        return None;
    }
    Some(&text[start as usize..pos as usize])
}


