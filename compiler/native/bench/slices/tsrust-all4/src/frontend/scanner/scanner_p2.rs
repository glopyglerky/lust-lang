//! Port of `scanner/scanner.go` lines 1237 to 2226: JSX and JSDoc scans,
//! identifiers, escapes, numbers and strings. Unit U2. The `Scanner` struct,
//! its state and the low-level character helpers (`char`, `char_at`,
//! `char_and_size`, `scan_ascii_while`, `error`, `error_at`, `scan`) are in
//! `scanner_p1.rs` (unit U1).
//!
//! Runes follow scanner_p1.rs: Go `rune` is `i32`, and `rune_to_char` maps
//! one to a `char` for the `stringutil` predicates. The token value is set
//! through `set_token_value` (interned `&'static str`, see scanner_p1.rs).

use crate::frontend::prelude::*;
use std::borrow::Cow;

use crate::scanner_util::push_js_string_rune;

use super::scanner_p1::{
    EscapeSequenceScanningFlags, IdentifierVariant, RUNE_SELF, 
    
};

/// Go `strconv.ParseInt(s, base, bitSize)` with the error ignored, as the
/// scanner does. The input holds only digits of `base`.
// PORT: on overflow Go returns the largest value for `bit_size` (and an
// error the scanner drops). This keeps that value. An empty or invalid input
// returns 0, as Go does.
fn go_parse_int(s: &str, base: u32, bit_size: u32) -> i64 {
    let max: i64 = if bit_size >= 64 {
        i64::MAX
    } else {
        (1i64 << (bit_size - 1)) - 1
    };
    if s.is_empty() {
        return 0;
    }
    let mut value: i64 = 0;
    for c in s.chars() {
        let Some(digit) = c.to_digit(base) else {
            return 0;
        };
        value = match value
            .checked_mul(base as i64)
            .and_then(|v| v.checked_add(digit as i64))
        {
            Some(v) if v <= max => v,
            _ => return max,
        };
    }
    value
}



/// The bytes that continue an ASCII identifier: letters, digits, `_`, `$`.
/// Bytes >= 0x80 are false, so a scan stops there like `scan_ascii_while`.
static ASCII_IDENT_PART: [bool; 256] = {
    let mut table = [false; 256];
    let mut i = 0;
    while i < 128 {
        let b = i as u8;
        table[i] = b.is_ascii_alphanumeric() || b == b'_' || b == b'$';
        i += 1;
    }
    table
};

/// A numeric fragment from `scan_number_fragment`: `prefix` followed by
/// `text[start..end]`. `prefix` holds the digits before the last separator
/// and stays empty (no allocation) when there is no separator.
pub(crate) struct NumberFragment {
    prefix: String,
    start: i32,
    end: i32,
}

impl NumberFragment {
    fn is_empty(&self) -> bool {
        self.prefix.is_empty() && self.start == self.end
    }

    /// Appends the fragment value to `out`.
    fn push_to(&self, text: &str, out: &mut String) {
        out.push_str(&self.prefix);
        out.push_str(&text[self.start as usize..self.end as usize]);
    }
}


#[cfg(test)]
mod tests {
    use super::{is_canonical_js_integer, js_number_string};

    #[test]
    fn canonical_integers_round_trip() {
        let samples = (0..1_000_000u64)
            .chain((0..1000u64).map(|i| 999_999_999_999_999 - i * 7_919_007_919))
            .map(|n| n.to_string())
            .chain(["00", "01", "1e3", "0x10", "", "1234567890123456"].map(String::from));
        for text in samples {
            if is_canonical_js_integer(&text) {
                assert_eq!(js_number_string(&text), text);
            }
        }
        assert!(!is_canonical_js_integer("01"));
        assert!(!is_canonical_js_integer("1234567890123456"));
    }
}
