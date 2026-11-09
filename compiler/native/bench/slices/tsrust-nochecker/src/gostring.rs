//! Go string helpers of `scanner_util.rs` (UTF-16 surrogates, the Go
//! string marker, `GoUnit`, Go byte offsets and compares). They live in
//! `goport_util` as the module `scanner_util`, so util files keep their
//! `crate::scanner_util::` paths. `ts_goport`'s `scanner_util.rs`
//! re-exports them.

use std::borrow::Cow;

/// First surrogate code point. It maps to `LONE_SURROGATE_UNIT_BASE`.
const SURROGATE_HIGH_START: u32 = 0xD800;
/// Go `SurrogateLowStart`.
pub const SURROGATE_LOW_START: u32 = 0xDC00;

/// Go `utf16.IsSurrogate`.
fn utf16_is_surrogate(ch: u32) -> bool {
    (0xD800..0xE000).contains(&ch)
}

// Go: stringutil/util.go:280 IsHighSurrogate
pub fn is_high_surrogate(ch: u32) -> bool {
    utf16_is_surrogate(ch) && ch < SURROGATE_LOW_START
}

// Go: stringutil/util.go:284 IsLowSurrogate
pub fn is_low_surrogate(ch: u32) -> bool {
    utf16_is_surrogate(ch) && ch >= SURROGATE_LOW_START
}

// Go: stringutil/util.go:288 IsSurrogate
pub fn is_surrogate(ch: u32) -> bool {
    utf16_is_surrogate(ch)
}

// Go: stringutil/util.go:292 SurrogatePairToCodePoint
pub fn surrogate_pair_to_code_point(high: u32, low: u32) -> u32 {
    // Go utf16.DecodeRune returns U+FFFD for an invalid pair.
    if (0xD800..0xDC00).contains(&high) && (0xDC00..0xE000).contains(&low) {
        (((high - 0xD800) << 10) | (low - 0xDC00)) + 0x10000
    } else {
        0xFFFD
    }
}

// PORT: a Go string is a byte string. Two kinds of Go bytes are not valid
// UTF-8, so a Rust `String` cannot hold them:
// - a lone surrogate (U+D800..U+DFFF), which Go `EncodeJSStringRune` stores
//   as its 3-byte WTF-8 form `ED A0..BF 80..BF`;
// - a source byte that is not valid UTF-8 (always 0x80..=0xFF), which Go
//   keeps unchanged and decodes as `utf8.RuneError` of size 1.
// This port stores a Go string in a "port form" with one escape. The marker
// M = GO_STRING_MARKER (U+FDD0, a Unicode noncharacter) starts a unit of two
// chars:
// - M + char(LONE_SURROGATE_UNIT_BASE + (cp - 0xD800)): the lone surrogate
//   `cp`. The second char is in U+10F800..U+10FFFF.
// - M + char(INVALID_BYTE_UNIT_BASE + b): the invalid byte `b`. The second
//   char is in U+10F780..U+10F7FF.
// - M + M: a real U+FDD0.
// Every other char is itself, including a real U+10F780..U+10FFFF char. So
// each Go string has one port form and each port form one Go string.
// `go_unit_at` reads one unit and `go_string_bytes` gives the Go bytes back.
// Where the port form comes from:
// - `vfs::decode_bytes` writes source text in it (`go_string_from_bytes`),
//   with one invalid byte unit for each byte that Go decodes as RuneError of
//   size 1. That includes each byte of a WTF-8 surrogate, as the Go scanner
//   reads them.
// - The scanner copies source text into token values. A string value fuses
//   the 3 invalid byte units of a WTF-8 surrogate into one lone surrogate
//   unit (`fuse_surrogate_bytes`). This fused form is the "value form": each
//   Go string has exactly one, so equal Go strings have equal value forms.
// - The OS gives file names, the working directory and arguments in the
//   value form (`vfs::go_string_from_os`), and takes the Go bytes back
//   (`vfs::os_path`).
// - A decoded JSON string is valid UTF-8, with each real U+FDD0 written
//   twice (`frontend::json`).
// - `push_js_string_rune` writes one rune.
// Go bytes leave the port only through file writes, OS paths, the process
// output and JSON strings, which all convert the port form
// (`go_string_bytes`, `frontend::json::append_json_quote`).
// A lone M that is not followed by M or a unit char (for example from Go
// `string(rune)`) reads as a real U+FDD0.
// Go works on the bytes of a string in places. The port does the same:
// - A unit is 6 or 7 bytes, not 3 or 1. Offsets in port text after a unit
//   differ from Go byte offsets. Callers use them to slice the same text.
//   `utf16_len` counts Go runes, so lines and columns match Go.
//   `go_byte_offset` and `port_byte_offset` convert an offset where Go
//   writes or reads it (build info diagnostics and declaration signatures).
//   `go_len` is Go `len`, for lengths that Go compares or limits.
// - Go joins strings by bytes, so joined invalid bytes can form a valid char
//   or a WTF-8 surrogate. `go_value` gives the value form of a join.
// - A byte search on the port form can match inside a unit, and a Go byte
//   search can match inside a char. `go_has_prefix`, `go_has_suffix` and
//   `go_slice` work on the Go bytes, as Go does.
pub const GO_STRING_MARKER: char = '\u{FDD0}';
const GO_STRING_MARKER_BYTES: &[u8] = "\u{FDD0}".as_bytes();
const LONE_SURROGATE_UNIT_BASE: u32 = 0x10F800;
const INVALID_BYTE_UNIT_BASE: u32 = 0x10F700;

/// One unit of the port form of a Go string (see `GO_STRING_MARKER`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoUnit {
    /// A char. A real U+FDD0 is `Char('\u{FDD0}')`.
    Char(char),
    /// A lone surrogate: Go bytes `ED A0..BF 80..BF`.
    Surrogate(u32),
    /// A Go byte that is not valid UTF-8 (0x80..=0xFF).
    InvalidByte(u8),
}

impl GoUnit {
    /// The number of Go bytes in the unit.
    pub fn go_len(self) -> usize {
        match self {
            GoUnit::Char(ch) => ch.len_utf8(),
            GoUnit::Surrogate(_) => 3,
            GoUnit::InvalidByte(_) => 1,
        }
    }

    /// Go `core.UTF16Len` of the unit's Go bytes. Go `range` reads each
    /// byte that is not valid UTF-8 as one RuneError, which is one UTF-16
    /// unit. The 3 bytes of a lone surrogate are 3 such bytes.
    pub fn go_utf16_len(self) -> usize {
        match self {
            GoUnit::Char(ch) => ch.len_utf16(),
            GoUnit::Surrogate(_) => 3,
            GoUnit::InvalidByte(_) => 1,
        }
    }

}

























// Go: stringutil/util.go:334 DecodeJSStringRune

/// Appends the port form of the Go rune `ch` (see `GO_STRING_MARKER`) to
/// `out`.
pub fn push_js_string_rune(out: &mut String, ch: u32) {
    if is_surrogate(ch) {
        out.push(GO_STRING_MARKER);
        // The unit base maps the surrogate range onto valid chars.
        out.push(
            char::from_u32(LONE_SURROGATE_UNIT_BASE + (ch - SURROGATE_HIGH_START))
                .unwrap_or(char::REPLACEMENT_CHARACTER),
        );
        return;
    }
    let c = char::from_u32(ch).unwrap_or(char::REPLACEMENT_CHARACTER);
    if c == GO_STRING_MARKER {
        out.push(GO_STRING_MARKER);
    }
    out.push(c);
}

// Go: stringutil/util.go:323 EncodeJSStringRune
// PORT: writes the port form (see GO_STRING_MARKER), not WTF-8.
pub fn encode_js_string_rune(ch: u32) -> String {
    let mut out = String::with_capacity(8);
    push_js_string_rune(&mut out, ch);
    out
}

// Go: stringutil/util.go:334 DecodeJSStringRune
// PORT: returns the Go `rune` as `u32`, because a lone surrogate is not a
// valid Rust `char`, and the size in `s`. It reads one unit of the port form
// (see GO_STRING_MARKER). An empty string gives U+FFFD with size 0, as
// `utf8.DecodeRuneInString`. `decode_go_js_string_rune` also gives the Go
// size.


// PORT: Go `strings.Compare` on the Go bytes of a string. Plain `str` order
// differs from Go in two places, because this port stores two Go byte forms
// in other ways (see GO_STRING_MARKER):
// - A lone surrogate unit is the 3-byte WTF-8 form `ED A0..BF 80..BF` in Go.
//   A real U+FDD0 unit (M + M) is `EF B7 90`.
// - An invalid byte unit is one byte 0x80..=0xFF in Go. This includes the
//   internal symbol name prefix `\xFE` (`ast::INTERNAL_SYMBOL_NAME_PREFIX`).
// The Go bytes of chars and lone surrogates are prefix free, so comparing the
// first different unit by its Go bytes gives the Go result. An invalid byte
// can be a prefix of another unit's bytes, so then the Go bytes of the rest
// of both strings are compared.





/// `go_byte_offset` and `port_byte_offset` for many offsets of one text,
/// with one scan for markers. LSP code lenses and references convert one
/// offset for each item; each call of the plain functions scans from the
/// text start.
pub struct GoOffsets {
    len: usize,
    /// The marker units in text order.
    units: Vec<GoOffsetUnit>,
}

struct GoOffsetUnit {
    /// The unit's port offset and size.
    at: usize,
    size: usize,
    /// The unit's Go offset and Go length.
    go_at: usize,
    go_len: usize,
}

impl GoOffsets {

    /// Reports whether the text has a marker unit. Without one, port and Go
    /// offsets are the same.
    pub fn has_units(&self) -> bool {
        !self.units.is_empty()
    }


}


// Go: stringutil/util.go:352 CombineSurrogatePairs
// CombineSurrogatePairs canonicalizes a JS-string value produced by
// concatenation, merging any adjacent high+low surrogate sentinel pair (as
// written by EncodeJSStringRune) into the single supplementary code point they
// represent. Strings without a lone-surrogate sentinel (the common case) are
// returned unchanged.
// PORT: the sentinel check looks for GO_STRING_MARKER.

