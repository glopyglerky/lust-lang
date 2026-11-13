//! Shared compiler primitives (`TextRange`, `JsString`, `SourceText`). This
//! was the `ts_core` crate.

use std::{fmt, ops::Range, sync::Arc};










/// A JavaScript string represented as UTF-16 code units. Unlike Rust `String`,
/// this preserves lone surrogates produced by escape sequences.
#[derive(Clone, Default, Eq, Hash, PartialEq)]
pub struct JsString(Vec<u16>);

impl JsString {

    #[must_use]
    pub fn from_units(units: Vec<u16>) -> Self {
        Self(units)
    }

    #[must_use]
    pub fn as_units(&self) -> &[u16] {
        &self.0
    }


    pub fn push_unit(&mut self, unit: u16) {
        self.0.push(unit);
    }

}

impl fmt::Debug for JsString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("JsString").field(&self.0).finish()
    }
}

/// A source position measured in UTF-8 bytes, matching the native compiler.
#[derive(Clone, Copy, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct TextPos(u32);

impl TextPos {
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl fmt::Debug for TextPos {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// A half-open source range measured in UTF-8 bytes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TextRange {
    pub start: TextPos,
    pub end: TextPos,
}

impl TextRange {
    #[must_use]
    pub const fn new(start: TextPos, end: TextPos) -> Self {
        Self { start, end }
    }

    #[must_use]
    pub const fn len(self) -> u32 {
        self.end.get() - self.start.get()
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start.get() == self.end.get()
    }
}

/// A compiler diagnostic independent of rendering and localization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub range: TextRange,
    pub code: Option<u32>,
    pub category: DiagnosticCategory,
    pub message: String,
}

/// TypeScript's diagnostic severity categories.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum DiagnosticCategory {
    Warning = 0,
    Error = 1,
    Suggestion = 2,
    Message = 3,
}

/// Bidirectional mapping between native UTF-8 byte offsets and JavaScript
/// UTF-16 code-unit offsets.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PositionMap {
    entries: Vec<PositionMapEntry>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PositionMapEntry {
    utf8_pos: usize,
    delta: usize,
}

impl PositionMap {
    #[must_use]
    pub fn new(text: &str) -> Self {
        let mut entries = Vec::new();
        let mut delta = 0;
        for (start, ch) in text.char_indices() {
            let utf8_size = ch.len_utf8();
            let utf16_size = ch.len_utf16();
            if utf8_size != utf16_size {
                delta += utf8_size - utf16_size;
                entries.push(PositionMapEntry {
                    utf8_pos: start + utf8_size,
                    delta,
                });
            }
        }
        Self { entries }
    }

    #[must_use]
    pub fn is_ascii_only(&self) -> bool {
        self.entries.is_empty()
    }


}

impl Diagnostic {
    #[must_use]
    pub fn new(range: TextRange, message: impl Into<String>) -> Self {
        Self {
            range,
            code: None,
            category: DiagnosticCategory::Error,
            message: message.into(),
        }
    }

    #[must_use]
    pub fn typescript(
        range: TextRange,
        code: u32,
        category: DiagnosticCategory,
        message: impl Into<String>,
    ) -> Self {
        Self {
            range,
            code: Some(code),
            category,
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Diagnostic, DiagnosticCategory, JsString, PositionMap, SourceText, TextPos, TextRange,
    };

    #[test]
    fn diagnostics_support_legacy_and_typescript_metadata() {
        let range = TextRange::new(TextPos::new(1), TextPos::new(2));
        let legacy = Diagnostic::new(range, "legacy");
        assert_eq!(legacy.code, None);
        assert_eq!(legacy.category, DiagnosticCategory::Error);

        let structured =
            Diagnostic::typescript(range, 9999, DiagnosticCategory::Warning, "structured");
        assert_eq!(structured.code, Some(9999));
        assert_eq!(structured.category, DiagnosticCategory::Warning);
    }

    #[test]
    fn source_text_preserves_invalid_bytes_and_offsets() {
        let source = SourceText::from_bytes(vec![b'a', 0x80, b'b', 0xf0, 0x9f]);
        assert_eq!(source.as_bytes(), &[b'a', 0x80, b'b', 0xf0, 0x9f]);
        assert_eq!(source.as_scannable_str().as_bytes(), b"a\x7fb\x7f\x7f");
        assert_eq!(source.invalid_byte_ranges(), &[1..2, 3..5]);
        assert_eq!(source.as_scannable_str().len(), source.len());
        assert!(source.as_str().is_none());
    }

    #[test]
    fn ranges_are_half_open() {
        let range = TextRange::new(TextPos::new(2), TextPos::new(7));
        assert_eq!(range.len(), 5);
        assert!(!range.is_empty());
    }

    #[test]
    fn maps_ascii_without_allocated_entries() {
        let map = PositionMap::new("const answer = 42");
        assert!(map.is_ascii_only());
        assert_eq!(map.utf8_to_utf16(10), 10);
        assert_eq!(map.utf16_to_utf8(10), 10);
    }

    #[test]
    fn maps_bmp_and_astral_characters() {
        let text = "aé😀b";
        let map = PositionMap::new(text);
        assert!(!map.is_ascii_only());
        assert_eq!(map.utf8_to_utf16(text.find('b').unwrap()), 4);
        assert_eq!(map.utf16_to_utf8(4), text.find('b').unwrap());
        for utf16_offset in 0..=5 {
            assert_eq!(
                map.utf8_to_utf16(map.utf16_to_utf8(utf16_offset)),
                utf16_offset
            );
        }
    }

    #[test]
    fn javascript_strings_preserve_lone_surrogates() {
        let value = JsString::from_units(vec![0xd83e, 0xdd80, 0xd800]);
        assert_eq!(value.as_units(), &[0xd83e, 0xdd80, 0xd800]);
        assert_eq!(value.to_string_lossy(), "🦀�");
    }
}
