//! Port of `scanner/regexp.go`: the regular expression body checker that
//! `Scanner.ReScanSlashToken` runs when it reports errors.
//!
//! PORT: Go strings are byte strings. The values that this checker returns
//! (`scanClassAtom`, `scanClassSetOperand`, ...) hold WTF-8 lone-surrogate
//! sentinels in Go. These values are only compared (empty check, length and
//! `DecodeJSStringRune`), so they are `Vec<u8>` here, and the private
//! `encode_js_string_rune_bytes`, `decode_js_string_rune_bytes` and
//! `decode_rune_in_bytes` helpers do the Go byte math on them. Lone
//! surrogates use the valid-UTF-8 port form from
//! `scanner_util::encode_js_string_rune` (see
//! `scanner_util::GO_STRING_MARKER`) instead of Go's WTF-8 bytes.
//!
//! Go `rune` values from `Scanner.char()` and `Scanner.charAt()` are `i32`
//! (a raw byte, or -1 at the end). `match rune(ch)` stands in for a Go
//! `switch ch` over ASCII cases.

use crate::flags_macros::{go_enum, go_flags};
use crate::frontend::prelude::*;

use super::scanner_p1::{
    EscapeSequenceScanningFlags, IdentifierVariant, RUNE_ERROR, 
    
};
use super::unicode_properties::*;

go_flags!(RegularExpressionFlags, i32 {
    NONE = 0; // regularExpressionFlagsNone
    HAS_INDICES = 1 << 0; // d
    GLOBAL = 1 << 1; // g
    IGNORE_CASE = 1 << 2; // i
    MULTILINE = 1 << 3; // m
    DOT_ALL = 1 << 4; // s
    UNICODE = 1 << 5; // u
    UNICODE_SETS = 1 << 6; // v
    STICKY = 1 << 7; // y
    ANY_UNICODE_MODE = (1 << 5) | (1 << 6); // regularExpressionFlagsUnicode | regularExpressionFlagsUnicodeSets
    MODIFIERS = (1 << 2) | (1 << 3) | (1 << 4); // regularExpressionFlagsIgnoreCase | regularExpressionFlagsMultiline | regularExpressionFlagsDotAll
});

// Go: scanner/regexp.go:34 charCodeToRegExpFlag
// PORT: Go map lookup `charCodeToRegExpFlag[ch]` -> `(flag, ok)` as `Option`.

// Go: scanner/regexp.go:45 regExpFlagToFirstAvailableLanguageVersion
// PORT: Go map lookup as `Option`.


go_enum!(ClassSetExpressionType, i32 {
    UNKNOWN = 0; // classSetExpressionTypeUnknown
    CLASS_UNION = 1; // classSetExpressionTypeClassUnion
    CLASS_INTERSECTION = 2; // classSetExpressionTypeClassIntersection
    CLASS_SUBTRACTION = 3; // classSetExpressionTypeClassSubtraction
});

// Go: scanner/regexp.go:66 groupNameReference
#[derive(Clone, Debug, Default)]
pub struct GroupNameReference {
    pub pos: i32,
    pub end: i32,
    pub name: String,
}

// Go: scanner/regexp.go:72 decimalEscapeValue
#[derive(Clone, Copy, Debug, Default)]
pub struct DecimalEscapeValue {
    pub pos: i32,
    pub end: i32,
    pub value: i32,
}

// Go: scanner/regexp.go:78 regExpParser

// PORT: Go builds `regExpParser` with a struct literal in
// `Scanner.ReScanSlashToken` (scanner.go:1210). This constructor takes the
// same fields and gives the other fields their Go zero values.


// Go: scanner/regexp.go:137 compareDecimalStrings


// ---------------------------------------------------------------------------
// Private Go runtime helpers (PORT: `unicode/utf8`, `unicode/utf16` and
// `stringutil` byte math on Go string bytes).
// ---------------------------------------------------------------------------



/// Go `utf8.DecodeRuneInString(s)`: `(RuneError, 0)` for empty input and
/// `(RuneError, 1)` for an invalid sequence.
fn decode_rune_in_bytes(s: &[u8]) -> (i32, i32) {
    if s.is_empty() {
        return (RUNE_ERROR, 0);
    }
    let head = &s[..s.len().min(4)];
    let valid = match std::str::from_utf8(head) {
        Ok(text) => text,
        // The prefix up to `valid_up_to` is valid UTF-8.
        Err(err) => std::str::from_utf8(&head[..err.valid_up_to()]).unwrap_or(""),
    };
    match valid.chars().next() {
        Some(ch) => (ch as i32, ch.len_utf8() as i32),
        None => (RUNE_ERROR, 1),
    }
}

/// Go `utf16.EncodeRune` through `stringutil.CodePointToSurrogatePair`.
fn code_point_to_surrogate_pair_i32(ch: i32) -> (i32, i32) {
    if !(0x10000..=0x10FFFF).contains(&ch) {
        return (RUNE_ERROR, RUNE_ERROR);
    }
    let r = ch - 0x10000;
    (0xD800 + ((r >> 10) & 0x3FF), 0xDC00 + (r & 0x3FF))
}

// Go: stringutil/util.go:323 EncodeJSStringRune
// PORT: byte form of `scanner_util::encode_js_string_rune`. A lone surrogate
// becomes the same port form unit (see `scanner_util::GO_STRING_MARKER`), so
// these bytes match what `scan_escape_sequence` returns. Other runes are their
// UTF-8 bytes. A lone U+FDD0 decodes as itself, so it needs no escape here.

// Go: stringutil/util.go:334 DecodeJSStringRune
// PORT: byte form of `scanner_util::decode_js_string_rune`. It reads one
// port form unit (see `scanner_util::GO_STRING_MARKER`) from the valid
// UTF-8 prefix, and keeps the Go `utf8.DecodeRuneInString` results for
// invalid bytes.
