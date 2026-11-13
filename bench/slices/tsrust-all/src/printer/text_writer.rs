//! Go `printer/textwriter.go`: the multi-line `EmitTextWriter`.

use crate::prelude::*;

/// Go `printer.textWriter`.
// Go: printer/textwriter.go:14 textWriter
#[derive(Clone, Debug, Default)]
pub struct TextWriter {
    new_line: String,
    indent_size: i32,
    builder: String,
    // PORT: Go keeps `lastWritten string`; its only reader is
    // `HasTrailingWhitespace`, which decodes the last rune. This keeps that
    // rune (`None` for ""), so a write does not copy the text.
    last_char: Option<char>,
    indent: i32,
    line_start: bool,
    line_count: i32,
    line_pos: i32,
    has_trailing_comment_state: bool,
    // PORT: not in Go. True when `builder[line_pos..]` has a byte >= 0x80.
    // When false, `get_column` is the byte count with no scan.
    line_non_ascii: bool,
}

impl TextWriter {
    // Go: printer/textwriter.go:97 updateLineCountAndPosFor
    // `s` is the text that the caller just pushed to `builder`.

    // Go: printer/textwriter.go:131 writeText

    // PORT: Go `w.builder.WriteString(getIndentString(w.indent, w.indentSize))`.
    // This pushes the same spaces from a static string, with no allocation.
    fn push_indent(&mut self) {
        // 64 spaces.
        const SPACES: &str = "                                                                ";
        // Go `strings.Repeat` panics on a negative count.
        let mut n = usize::try_from(self.indent * self.indent_size)
            .expect("strings: negative Repeat count");
        while n > 0 {
            let k = n.min(SPACES.len());
            self.builder.push_str(&SPACES[..k]);
            n -= k;
        }
    }

    // Go: printer/textwriter.go:161 writeLineRaw
    fn write_line_raw(&mut self) {
        self.builder.push_str(&self.new_line);
        self.last_char = self.new_line.chars().next_back();
        self.line_count += 1;
        self.line_pos = self.builder.len() as i32;
        self.line_non_ascii = false;
        self.line_start = true;
        self.has_trailing_comment_state = false;
    }
}


const DEFAULT_INDENT_SIZE: i32 = 4;

// Go: printer/textwriter.go:119 GetDefaultIndentSize
// GetDefaultIndentSize returns the default indent size (4 spaces) used when no specific indent size is configured.
#[must_use]
pub fn get_default_indent_size() -> i32 {
    DEFAULT_INDENT_SIZE
}

// Go: printer/textwriter.go:123 getIndentString
// PORT: `TextWriter` writes its indent with `push_indent`; other callers use this.
pub fn get_indent_string(indent: i32, indent_size: i32) -> String {
    if indent == 0 {
        return String::new();
    }
    // TODO: This is cached in tsc - should it be cached here?
    // Go `strings.Repeat` panics on a negative count.
    " ".repeat(usize::try_from(indent * indent_size).expect("strings: negative Repeat count"))
}

// Go: printer/textwriter.go:218 NewTextWriter
// PORT: Go returns the `EmitTextWriter` interface; this returns the concrete
// writer, which callers use as `&mut dyn EmitTextWriter`.

// PORT: not a Go function. Go code in package printer writes the struct
// literal `textWriter{newLine: newLine, indentSize: indentSize}` (for example
// `NewChangeTrackerWriter`). The fields are private to this file, so this
// builds that literal. Unlike `new_text_writer`, it keeps an indent size of 0
// and does not call `clear`.
#[must_use]
pub(crate) fn new_text_writer_literal(new_line: &str, indent_size: i32) -> TextWriter {
    TextWriter {
        new_line: new_line.to_string(),
        indent_size,
        ..TextWriter::default()
    }
}

/// PORT: not a Go function. Counts the ECMAScript line breaks in `s` and
/// finds the start of its last line (0 when there is no break). It uses the
/// rules of `compute_ecma_line_starts` (`\n`, `\r`, `\r\n` as one break,
/// U+2028, U+2029): the count is that Vec's length minus 1 and the start is
/// its last value.
fn scan_line_breaks(s: &str) -> (i32, i32) {
    let bytes = s.as_bytes();
    let mut breaks = 0;
    let mut line_start = 0usize;
    // U+2028 is E2 80 A8 and U+2029 is E2 80 A9. 0xE2 is never a UTF-8
    // continuation byte, so each hit starts a char.
    for i in crate::memchr::memchr3_iter(b'\n', b'\r', 0xE2, bytes) {
        if i < line_start {
            // The `\n` of a `\r\n` pair, which the `\r` already counted.
            continue;
        }
        let next = match bytes[i] {
            b'\n' => i + 1,
            b'\r' if bytes.get(i + 1) == Some(&b'\n') => i + 2,
            b'\r' => i + 1,
            _ if bytes.get(i + 1) == Some(&0x80)
                && matches!(bytes.get(i + 2), Some(0xA8 | 0xA9)) =>
            {
                i + 3
            }
            _ => continue,
        };
        breaks += 1;
        line_start = next;
    }
    (breaks, line_start as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_line_breaks_matches_compute_ecma_line_starts() {
        for s in [
            "",
            "abc",
            "a\r\nb",
            "a\rb",
            "a\r",
            "\r\n\r\n",
            "\n\r",
            "a\u{2028}b\u{2029}c",
            "\u{2028}",
            "caf\u{e9} \u{2020}\u{2030}\u{20ac}\u{1f600}",
            "x\r\ny\rz\u{2029}\u{e9}\n\u{2028}end\r\n",
        ] {
            let starts = compute_ecma_line_starts(s);
            let expected = (starts.len() as i32 - 1, *starts.last().unwrap());
            assert_eq!(scan_line_breaks(s), expected, "{s:?}");
        }
    }
}
