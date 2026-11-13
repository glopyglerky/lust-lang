//! Port of Effect-TS/tsgo `internal/rules/duplicate_package.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

/// duplicatePackageDiag holds pre-computed diagnostic info for a single duplicated package name.
struct DuplicatePackageDiag {
    package_name: String,
    /// e.g. "1.0.0 @ /path/a, 2.0.0 @ /path/b"
    details: String,
    config_value: String,
}




/// Go `encoding/json` (v1) `Marshal` of a `[]string`: a JSON array with Go's
/// default string escaping (HTML-safe: `<`, `>` and `&` become `\u003c`,
/// `\u003e` and `\u0026`; U+2028 and U+2029 are escaped).
// PORT: no Go function; it inlines `encodeState.string` with escapeHTML.
fn encoding_json_marshal_strings(values: &[String]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::from("[");
    for (i, value) in values.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push('"');
        for ch in value.chars() {
            match ch {
                '\\' => out.push_str("\\\\"),
                '"' => out.push_str("\\\""),
                '\u{08}' => out.push_str("\\b"),
                '\u{0C}' => out.push_str("\\f"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                '<' | '>' | '&' => {
                    let b = ch as u8;
                    out.push_str("\\u00");
                    out.push(HEX[(b >> 4) as usize] as char);
                    out.push(HEX[(b & 0xF) as usize] as char);
                }
                '\u{2028}' => out.push_str("\\u2028"),
                '\u{2029}' => out.push_str("\\u2029"),
                c if (c as u32) < 0x20 => {
                    let b = c as u8;
                    out.push_str("\\u00");
                    out.push(HEX[(b >> 4) as usize] as char);
                    out.push(HEX[(b & 0xF) as usize] as char);
                }
                c => out.push(c),
            }
        }
        out.push('"');
    }
    out.push(']');
    out
}
