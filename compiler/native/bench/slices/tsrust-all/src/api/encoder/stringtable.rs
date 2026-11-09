//! Port of Go `api/encoder/stringtable.go`.
//!
//! PORT: Go `int` byte lengths and offsets in this file are `usize` (they
//! index byte buffers). Go string slices and comparisons work on bytes, so
//! the port compares bytes and never slices a `&str`.
//!
//! PORT: the table holds Go bytes, not the port form (see
//! `GO_STRING_MARKER`), so the client reads the same bytes and offsets as
//! from Go: a lone surrogate is its 3 WTF-8 bytes and an invalid source byte
//! is 1 byte. Node `pos` and `end` are port offsets in the file text, so
//! `add` changes them to Go offsets first.

use std::borrow::Cow;

use crate::api::encoder::prelude::*;

// Go: api/encoder/stringtable.go:9 stringTable

// Go: api/encoder/stringtable.go:16 newStringTable




#[cfg(test)]
mod tests {
    use super::*;

    /// The string data section of `table.encode()`.
    fn string_data(table: &StringTable) -> Vec<u8> {
        table.encode()[table.offsets.len() * 4..].to_vec()
    }

    // The native-preview API test "unicode escapes" has the file
    // `"\ud800a\udc00"`. Go stores the value in 7 bytes (two 3-byte WTF-8
    // surrogates and "a"), after the 15 bytes of the file text.
    #[test]
    fn lone_surrogates_are_wtf8() {
        let file_text = r#""\ud800a\udc00""#;
        let mut table = new_string_table(file_text, 2);
        let text = encode_js_string_rune(0xD800) + "a" + &encode_js_string_rune(0xDC00);
        assert_eq!(table.add(&text, SyntaxKind::StringLiteral, 0, 15), 0);
        assert_eq!(table.offsets, [15, 22]);
        let mut want = file_text.as_bytes().to_vec();
        want.extend_from_slice(b"\xED\xA0\x80a\xED\xB0\x80");
        assert_eq!(string_data(&table), want);
        assert_eq!(table.string_length(), 22);
        assert_eq!(table.encoded_length(), table.encode().len());
    }

    // A file with an invalid byte and a real U+FDD0. Go writes their 1 and
    // 3 bytes, and a string after them points into the file at Go offsets.
    #[test]
    fn file_units_use_go_offsets() {
        let file_text: &'static str = go_string_from_bytes(b"\xFF\xEF\xB7\x90 ab".to_vec()).leak();
        let mut table = new_string_table(file_text, 3);
        let port_end = file_text.len() as i32;
        assert_eq!(port_end, 7 + 6 + 3);
        assert_eq!(table.add(file_text, SyntaxKind::SourceFile, 0, port_end), 0);
        assert_eq!(table.add("ab", SyntaxKind::Identifier, 13, port_end), 2);
        // "\u{FDD0}" has no match in the file text, so it goes after it.
        let fdd0 = go_string_from_bytes(b"\xEF\xB7\x90".to_vec());
        assert_eq!(table.add(&fdd0, SyntaxKind::Unknown, 0, 0), 4);
        assert_eq!(table.offsets, [0, 7, 5, 7, 7, 10]);
        assert_eq!(
            string_data(&table),
            b"\xFF\xEF\xB7\x90 ab\xEF\xB7\x90".to_vec()
        );
    }
}
