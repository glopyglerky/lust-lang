//! Port of Go `parser/jsdoc.go`: the JSDoc parts of `Parser` (unit U10).
//!
//! Every function is an `impl Parser` method, except the package-level
//! helpers. The `Parser` struct and the shared helpers (`mark`, `rewind`,
//! `look_ahead`, `finish_node`, `new_node_list`, the error helpers) are in
//! the other parser units.
//!
//! PORT: Go reuses slices across calls (`nodeSliceArena`, `stringSliceArena`,
//! `jsdocCommentsSpace`, `jsdocCommentRangesSpace`, `jsdocTagCommentsSpace`,
//! `jsdocTagCommentsPartsSpace`). They only save allocations. Here each call
//! uses a new `Vec`, and the parser fields are not used.

use crate::frontend::prelude::*;

// Go: jsdoc.go:13 init
// PORT: Go registers `parseJSDocForNode` with `ast.SetParseJSDocForNode`.
// Rust has no init hooks. The AST JSDoc lookup calls `parse_js_doc_for_node`
// directly.

// Go: jsdoc.go:19 parseJSDocForNode

/// Go `jsdocState`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum JsdocState {
    BeginningOfLine,
    SawAsterisk,
    SavingComments,
    SavingBackticks,
}

/// Go `propertyLikeParse` (a bit set).
pub type PropertyLikeParse = i32;

pub const PROPERTY_LIKE_PARSE_PROPERTY: PropertyLikeParse = 1 << 0;
pub const PROPERTY_LIKE_PARSE_PARAMETER: PropertyLikeParse = 1 << 1;
pub const PROPERTY_LIKE_PARSE_CALLBACK_PARAMETER: PropertyLikeParse = 1 << 2;



/// The size of an invalid byte unit of the port form: the marker U+FDD0 (3
/// bytes) and a char of U+10F780..U+10F7FF (4 bytes).
const INVALID_BYTE_UNIT_LEN: i32 = 7;


// Go: jsdoc.go:380 removeLeadingNewlines
// PORT: Go returns the subslice `comments[i:]`; this drops the prefix in place.

// Go: jsdoc.go:388 trimEnd

// Go: jsdoc.go:392 removeTrailingWhitespace
// PORT: Go returns the subslice `comments[:end]`; this truncates in place.

// Go: jsdoc.go:776 isJSDocLinkTag
fn is_js_doc_link_tag(kind: &str) -> bool {
    kind == "link" || kind == "linkcode" || kind == "linkplain"
}

// Go: jsdoc.go:818 isObjectOrObjectArrayTypeReference

// Go: jsdoc.go:1174 textsEqual

#[cfg(test)]
mod tests {
    use super::jsdoc_text_cut;
    use crate::scanner_util::go_string_from_bytes;

    /// The cut of a closed comment keeps no bytes and allocates nothing
    /// (the R159 reviewer: it ran 2 walks and 1 allocation per JSDoc
    /// comment). A cut inside a char keeps Go's bytes of it.
    #[test]
    fn jsdoc_text_cut_allocates_only_for_a_cut_char() {
        let text = "/** a */";
        let (cut, kept) = jsdoc_text_cut(text, text.len());
        assert_eq!((cut, kept.capacity()), (text.len() - 2, 0));
        // `日` is 3 Go bytes: the cut keeps its first byte.
        let text = "/** \u{65E5}";
        assert_eq!(jsdoc_text_cut(text, text.len()), (4, vec![0xE6]));
        // The invalid byte FF and `y` (1 Go byte each, 8 port bytes): the cut
        // is at the unit, with nothing kept.
        let text = go_string_from_bytes(b"/** \xFFy".to_vec());
        let (cut, kept) = jsdoc_text_cut(&text, text.len());
        assert_eq!((cut, kept.capacity()), (4, 0));
    }
}
