//! Port of Go `stringutil/compare.go` (whole file) and
//! `stringutil/util.go:256-273` (`TruncateByRunes`) for the language service.
//!
//! PORT: Go strings can hold invalid UTF-8; Rust `&str` cannot. Go decodes
//! runes with `utf8.DecodeRuneInString`, which gives U+FFFD for an invalid
//! byte. Over a `&str` that case only happens where Go slices bytes inside a
//! rune (`HasPrefix`, `HasSuffix`), so those compare byte slices with
//! `equal_fold` below.

use crate::frontend::prelude::*;

// Go: stringutil/compare.go:9 EquateStringCaseInsensitive

// Go: stringutil/compare.go:15 EquateStringCaseSensitive
pub fn equate_string_case_sensitive(a: &str, b: &str) -> bool {
    a == b
}

// Go: stringutil/compare.go:19 GetStringEqualityComparer

// Go: stringutil/compare.go:26 Comparison
pub type Comparison = i32;

// Go: stringutil/compare.go:29 ComparisonLessThan
pub const COMPARISON_LESS_THAN: Comparison = -1;
// Go: stringutil/compare.go:30 ComparisonEqual
pub const COMPARISON_EQUAL: Comparison = 0;
// Go: stringutil/compare.go:31 ComparisonGreaterThan
pub const COMPARISON_GREATER_THAN: Comparison = 1;

// Go: stringutil/compare.go:34 CompareStringsCaseInsensitive

// Go: stringutil/compare.go:63 CompareStringsCaseSensitive
// PORT: Go `strings.Compare` is a byte-wise comparison, like `str::cmp`.
pub fn compare_strings_case_sensitive(a: &str, b: &str) -> Comparison {
    a.cmp(b) as Comparison
}

// Go: stringutil/compare.go:67 GetStringComparer

// Go: stringutil/compare.go:74 HasPrefix

// Go: stringutil/compare.go:84 HasSuffix

// Go: stringutil/compare.go:94 HasPrefixAndSuffixWithoutOverlap

// Go: stringutil/compare.go:102 CompareStringsCaseInsensitiveThenSensitive

// Go: stringutil/compare.go:121 CompareStringsCaseInsensitiveEslintCompatible

// Go: stringutil/util.go:256 TruncateByRunes
pub fn truncate_by_runes(str: &str, max_length: i32) -> String {
    if (str.len() as i32) < max_length {
        return str.to_string();
    }
    if max_length <= 0 {
        return String::new();
    }
    let mut rune_count: i32 = 0;
    // Go: for i := range str (byte index of each rune start)
    for (i, _) in str.char_indices() {
        rune_count += 1;
        if rune_count > max_length {
            return str[..i].to_string();
        }
    }
    str.to_string()
}

// ---------------------------------------------------------------------------
// Go standard library helpers (`unicode`, `unicode/utf8`, `strings`).
// ---------------------------------------------------------------------------

/// Go `utf8.DecodeRuneInString` on a valid `&str`: `(RuneError, 0)` when empty.
fn decode_rune_in_string(s: &str) -> (char, usize) {
    match s.chars().next() {
        Some(c) => (c, c.len_utf8()),
        None => (char::REPLACEMENT_CHARACTER, 0),
    }
}

/// Go `utf8.DecodeRune` on bytes. An invalid or truncated sequence gives
/// U+FFFD with width 1, like Go. `s` is not empty at the call sites.
fn decode_rune_bytes(s: &[u8]) -> (char, usize) {
    let head = &s[..s.len().min(4)];
    let valid = match std::str::from_utf8(head) {
        Ok(v) => v,
        // `valid_up_to` marks the valid UTF-8 prefix.
        Err(e) => std::str::from_utf8(&head[..e.valid_up_to()]).unwrap_or(""),
    };
    match valid.chars().next() {
        Some(c) => (c, c.len_utf8()),
        None => (char::REPLACEMENT_CHARACTER, 1),
    }
}





