//! Port of `printer/utilities.go`.

use crate::prelude::*;

use crate::flags_macros::go_flags;
use crate::printer::text_writer::get_default_indent_size;

// Go: printer/utilities.go:18 getLiteralTextFlags
go_flags!(GetLiteralTextFlags, i32 {
    NONE = 0; // getLiteralTextFlagsNone
    NEVER_ASCII_ESCAPE = 1 << 0; // getLiteralTextFlagsNeverAsciiEscape
    JSX_ATTRIBUTE_ESCAPE = 1 << 1; // getLiteralTextFlagsJsxAttributeEscape
    TERMINATE_UNTERMINATED_LITERALS = 1 << 2; // getLiteralTextFlagsTerminateUnterminatedLiterals
    ALLOW_NUMERIC_SEPARATOR = 1 << 3; // getLiteralTextFlagsAllowNumericSeparator
});

// Go: printer/utilities.go:28 QuoteChar
/// Go `type QuoteChar rune`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct QuoteChar(pub char);

impl QuoteChar {
    pub const SINGLE_QUOTE: Self = Self('\''); // QuoteCharSingleQuote
    pub const DOUBLE_QUOTE: Self = Self('"'); // QuoteCharDoubleQuote
    pub const BACKTICK: Self = Self('`'); // QuoteCharBacktick
}

// Go: printer/utilities.go:36 jsxEscapedCharsMap
// PORT: the Go map becomes a match.
fn jsx_escaped_chars_map(ch: char) -> Option<&'static str> {
    match ch {
        '"' => Some("&quot;"),
        '\'' => Some("&apos;"),
        _ => None,
    }
}

// Go: printer/utilities.go:41 escapedCharsMap
// PORT: the Go map becomes a match.
fn escaped_chars_map(ch: char) -> Option<&'static str> {
    Some(match ch {
        '\t' => "\\t",
        '\u{000B}' => "\\v",
        '\u{000C}' => "\\f",
        '\u{0008}' => "\\b",
        '\r' => "\\r",
        '\n' => "\\n",
        '\\' => "\\\\",
        '"' => "\\\"",
        '\'' => "\\'",
        '`' => "\\`",
        '$' => "\\$",            // when quoteChar == '`'
        '\u{2028}' => "\\u2028", // lineSeparator
        '\u{2029}' => "\\u2029", // paragraphSeparator
        '\u{0085}' => "\\u0085", // nextLine
        _ => return None,
    })
}

// Go: printer/utilities.go:58 encodeJsxCharacterEntity
fn encode_jsx_character_entity(b: &mut String, char_code: u32) {
    let hex_char_code = format!("{char_code:X}");
    b.push_str("&#x");
    b.push_str(&hex_char_code);
    b.push(';');
}

// Go: printer/utilities.go:65 encodeUtf16EscapeSequence
fn encode_utf16_escape_sequence(b: &mut String, char_code: u32) {
    let hex_char_code = format!("{char_code:X}");
    b.push_str("\\u");
    for _ in hex_char_code.len()..4 {
        b.push('0');
    }
    b.push_str(&hex_char_code);
}

// Go: printer/utilities.go:77 escapeStringWorker
// Based heavily on the abstract 'Quote'/'QuoteJSONString' operation from ECMA-262 (24.3.2.2),
// but augmented for a few select characters (e.g. lineSeparator, paragraphSeparator, nextLine)
// Note that this doesn't actually wrap the input in double quotes.
// PORT: `s` is the port form of the Go string (see
// `scanner_util::GO_STRING_MARKER`). `decode_go_js_string_rune` reads one
// unit and also gives its Go size, so the invalid byte check is Go's. A lone
// surrogate unit prints as `\uD800` like Go. Text that this worker copies
// keeps the port form, and the file write writes its Go bytes. `ch` is the
// `char` form of `code`. For a surrogate it is U+FFFD, which matches no `ch`
// case, the same as the surrogate rune in Go.

/// True when the loop of `escape_string_worker` copies `byte` as it is,
/// whatever the flags and the bytes around it: printable ASCII (and DEL)
/// other than a backslash, the quote char and, in a template, `$`.
fn copies_as_is(byte: u8, quote_char: QuoteChar) -> bool {
    (0x20..0x80).contains(&byte)
        && byte != b'\\'
        && char::from(byte) != quote_char.0
        && !(byte == b'$' && quote_char == QuoteChar::BACKTICK)
}


// Go: printer/utilities.go:178 EscapeString

// Go: printer/utilities.go:185 escapeNonAsciiString

// Go: printer/utilities.go:192 escapeJsxAttributeString

// Go: printer/utilities.go:199 canUseOriginalText

// Go: printer/utilities.go:227 getLiteralText

// Go: printer/utilities.go:341 isNotPrologueDirective

// Go: printer/utilities.go:345 RangeIsOnSingleLine

// Go: printer/utilities.go:349 RangeStartPositionsAreOnSameLine

// Go: printer/utilities.go:357 rangeEndPositionsAreOnSameLine

// Go: printer/utilities.go:361 rangeStartIsOnSameLineAsRangeEnd

// Go: printer/utilities.go:365 rangeEndIsOnSameLineAsRangeStart

// Go: printer/utilities.go:369 getStartPositionOfRange

// Go: printer/utilities.go:376 PositionsAreOnSameLine

// Go: printer/utilities.go:380 GetLinesBetweenPositions

// Go: printer/utilities.go:397 getLinesBetweenRangeEndAndRangeStart

// Go: printer/utilities.go:402 getLinesBetweenPositionAndPrecedingNonWhitespaceCharacter

// Go: printer/utilities.go:408 getLinesBetweenPositionAndNextNonWhitespaceCharacter

// Go: printer/utilities.go:413 getPreviousNonWhitespacePosition

// Go: printer/utilities.go:422 siblingNodePositionsAreComparable

// Go: printer/utilities.go:443 getContainingNodeArray

// Go: printer/utilities.go:553 canHaveDecorators
// PORT: printer-local; differs from ast::can_have_decorators, so it stays
// private to avoid a glob-import clash.

// Go: printer/utilities.go:567 originalNodesHaveSameParent

/// Go `interface{ End() int }` as used by `tryGetEnd` and `greatestEnd`.
// PORT: Go switches on the dynamic type and panics on other types; here the
// trait limits callers at compile time. Go `*core.TextRange` is
// `Option<TextRange>`.
pub(crate) trait EndLike {
    fn try_get_end(&self) -> (i32, bool);
}




impl EndLike for Option<TextRange> {
    fn try_get_end(&self) -> (i32, bool) {
        match self {
            Some(v) => (v.end(), true),
            None => (0, false),
        }
    }
}

impl EndLike for TextRange {
    fn try_get_end(&self) -> (i32, bool) {
        (self.end(), true)
    }
}

// Go: printer/utilities.go:578 tryGetEnd
pub(crate) fn try_get_end(node: &dyn EndLike) -> (i32, bool) {
    // avoid using reflect (via core.IsNil) for common cases
    node.try_get_end()
}

// Go: printer/utilities.go:605 greatestEnd
pub(crate) fn greatest_end(mut end: i32, nodes: &[&dyn EndLike]) -> i32 {
    for i in (0..nodes.len()).rev() {
        let node = nodes[i];
        let (node_end, ok) = try_get_end(node);
        if ok && end < node_end {
            end = node_end;
        }
    }
    end
}

// Go: printer/utilities.go:614 skipSynthesizedParentheses

// Go: printer/utilities.go:621 isNewExpressionWithoutArguments

// Go: printer/utilities.go:625 isBinaryOperation

// Go: printer/utilities.go:631 mixingBinaryOperatorsRequiresParentheses
pub(crate) fn mixing_binary_operators_requires_parentheses(a: SyntaxKind, b: SyntaxKind) -> bool {
    if a == SyntaxKind::QuestionQuestionToken {
        return b == SyntaxKind::AmpersandAmpersandToken || b == SyntaxKind::BarBarToken;
    }
    if b == SyntaxKind::QuestionQuestionToken {
        return a == SyntaxKind::AmpersandAmpersandToken || a == SyntaxKind::BarBarToken;
    }
    false
}

// Go: printer/utilities.go:641 isImmediatelyInvokedFunctionExpressionOrArrowFunction

// Go: printer/utilities.go:650 hasLeadingHash
fn has_leading_hash(text: &str) -> bool {
    !text.is_empty() && text.as_bytes()[0] == b'#'
}

// Go: printer/utilities.go:654 removeLeadingHash
pub(crate) fn remove_leading_hash(text: &str) -> &str {
    if has_leading_hash(text) {
        &text[1..]
    } else {
        text
    }
}

// Go: printer/utilities.go:662 ensureLeadingHash
pub(crate) fn ensure_leading_hash(text: &str) -> String {
    if has_leading_hash(text) {
        text.to_string()
    } else {
        format!("#{text}")
    }
}

// Go: printer/utilities.go:670 FormatGeneratedName
pub fn format_generated_name(private_name: bool, prefix: &str, base: &str, suffix: &str) -> String {
    let name = format!(
        "{}{}{}",
        remove_leading_hash(prefix),
        remove_leading_hash(base),
        remove_leading_hash(suffix)
    );
    if private_name {
        return ensure_leading_hash(&name);
    }
    name
}

// Go: printer/utilities.go:678 isASCIIWordCharacter

// Go: printer/utilities.go:682 makeIdentifierFromModuleName
// PORT: Go reads the bytes of the Go string. `module_name` is a port form
// (see `scanner_util::GO_STRING_MARKER`), so this reads its Go bytes. Each
// byte that is not ASCII becomes '_', so the kept bytes are ASCII.

// Go: printer/utilities.go:706 findSpanEndWithEmitContext

// Go: printer/utilities.go:714 findSpanEnd
pub(crate) fn find_span_end<T: Copy>(array: &[T], test: impl Fn(T) -> bool, start: i32) -> i32 {
    let mut i = start;
    while (i as usize) < array.len() && test(array[i as usize]) {
        i += 1;
    }
    i
}


// Go: printer/utilities.go:722 skipWhiteSpaceSingleLine
// PORT: byte positions are `usize` in these private scanning helpers.

// Go: printer/utilities.go:732 matchWhiteSpaceSingleLine

// Go: printer/utilities.go:738 matchRune

// Go: printer/utilities.go:747 matchString

// Go: printer/utilities.go:767 matchQuotedString

// Go: printer/utilities.go:795 IsRecognizedTripleSlashComment
// /// <reference path="..." />
// /// <reference types="..." />
// /// <reference lib="..." />
// /// <reference no-default-lib="..." />
// /// <amd-dependency path="..." />
// /// <amd-module />

// Go: printer/utilities.go:852 isJSDocLikeText
// PORT: Go `comment.Len()` counts Go bytes (`comment_go_len_at_least`).
// The bytes at `pos+2` and `pos+3` follow the ASCII `/*`: a byte there is
// ASCII in the port form exactly when it is in Go, so the compares agree.

// Go: printer/utilities.go:859 IsPinnedComment
// PORT: Go `comment.Len() > 5` counts Go bytes (`comment_go_len_at_least`).


// Go: printer/utilities.go:865 calculateIndent

// Go: printer/utilities.go:894 lineCharacterCache
// lineCharacterCache provides cached line/character lookups for a source file,
// optimized for monotonically increasing positions (e.g., during source map emit).
//
// When positions increase within the same line, only the delta between the last
// position and the new position needs to be scanned for UTF-16 code unit counts,
// turning what would be O(n²) into O(n) for long lines.
//
// Character offsets are measured in UTF-16 code units per the source map specification.


// Go: printer/utilities.go:903 newLineCharacterCache
// PORT: a node source must be a parsed source file; the printer passes the
// original file of a transformed one (see `set_source_map_source`).


#[cfg(test)]
mod tests {
    use super::*;

    // Go prints `%#v` of the parent's Kind in this panic
    // (printer/utilities.go:457 `panic(fmt.Sprintf("Unexpected TypeParameter
    // parent: %#v", parent.Kind))`). Kind is an int16 with no `GoString`
    // method, so Go prints the number: 201 is `KindMappedType` at pin N
    // (ast/kind_generated.go).
    #[test]
    fn unexpected_type_parameter_parent_panic_prints_the_kind_number() {
        use crate::frontend::parser::{SourceFileParseOptions, parse_source_file};
        use std::panic::AssertUnwindSafe;
        let opts = SourceFileParseOptions {
            file_name: "/mapped.ts".to_string(),
            ..Default::default()
        };
        let root = parse_source_file(&opts, "type M = { [K in string]: K };", ScriptKind::TS).root;
        let alias = root.statements().iter().next().expect("the alias");
        let type_parameter = alias.type_().type_parameter();
        let payload = std::panic::catch_unwind(AssertUnwindSafe(|| {
            get_containing_node_array(type_parameter)
        }))
        .expect_err("no panic");
        assert_eq!(
            payload.downcast_ref::<String>().map(String::as_str),
            Some("Unexpected TypeParameter parent: 201")
        );
    }

    /// The escape with the skipped start (`escape_string_worker`) gives the
    /// same text as the Go loop from the first byte, for every quote char,
    /// flag set and ASCII byte, alone and next to the bytes that the loop
    /// reads ahead.
    #[test]
    fn skipped_start_gives_the_same_text() {
        let quotes = [
            QuoteChar::SINGLE_QUOTE,
            QuoteChar::DOUBLE_QUOTE,
            QuoteChar::BACKTICK,
        ];
        let flag_sets = [
            GetLiteralTextFlags::NONE,
            GetLiteralTextFlags::NEVER_ASCII_ESCAPE,
            GetLiteralTextFlags::JSX_ATTRIBUTE_ESCAPE,
            GetLiteralTextFlags::JSX_ATTRIBUTE_ESCAPE | GetLiteralTextFlags::NEVER_ASCII_ESCAPE,
        ];
        let mut texts: Vec<String> = [
            "",
            "plain text",
            "a\u{e9}b",
            "a\u{2028}b",
            "\u{1F600}",
            "${x}",
            "a$b",
            "a\r\nb",
            "\u{0}1",
        ]
        .iter()
        .map(ToString::to_string)
        .collect();
        for byte in 0u8..0x80 {
            let c = char::from(byte);
            texts.push(c.to_string());
            texts.push(format!("ab{c}cd"));
            for next in ['{', '\n', '0', '\u{e9}'] {
                texts.push(format!("x{c}{next}"));
            }
        }
        for text in &texts {
            for quote in quotes {
                for flags in flag_sets {
                    let mut full = String::new();
                    escape_string_worker_from(text, quote, flags, &mut full, 0);
                    let mut skipped = String::new();
                    escape_string_worker(text, quote, flags, &mut skipped);
                    assert_eq!(skipped, full, "{text:?} {quote:?} {flags:?}");
                }
            }
        }
    }

    /// `isJSDocLikeText` and `IsPinnedComment` compare Go `comment.Len()`,
    /// the Go bytes (printer/utilities.go:852 and :859). An invalid byte is
    /// 1 Go byte and 7 port bytes, so `/**` + FF is 4 Go bytes and not JSDoc
    /// like, and `/*!` + FF + `x` is 5 and not pinned. Declaration emit of a
    /// file that ends in `/**` + FF wrote the comment where Go writes none.
    #[test]
    fn comment_checks_count_go_bytes() {
        let comment = |go: &[u8]| {
            let text = crate::scanner_util::go_string_from_bytes(go.to_vec());
            let range = CommentRange {
                text_range: TextRange::new(0, text.len() as i32),
                kind: SyntaxKind::MultiLineCommentTrivia,
                has_trailing_new_line: false,
            };
            (text, range)
        };
        for (go, jsdoc_like, pinned) in [
            (&b"/**\xFF"[..], false, false),
            (b"/**\xFF*", true, false),
            (b"/*!\xFFx", false, false),
            (b"/*!\xFFx*", false, true),
            (b"/*!\xED\xA0\x80", false, true),
            (b"/**\xEF\xB7\x90", true, false),
        ] {
            let (text, range) = comment(go);
            assert_eq!(is_js_doc_like_text(&text, range), jsdoc_like, "{go:?}");
            assert_eq!(is_pinned_comment(&text, range), pinned, "{go:?}");
        }
    }

    /// A text source for `LineCharacterCache`.
    struct TextSource {
        text: String,
        line_map: Vec<i32>,
    }

    impl crate::sourcemap::source::Source for TextSource {
        fn text(&self) -> &str {
            &self.text
        }
        fn file_name(&self) -> &str {
            "parserSkippedTokens16.ts"
        }
        fn ecma_line_map(&self) -> &[i32] {
            &self.line_map
        }
    }

    /// A source map position inside a char counts as in Go
    /// (`printer/utilities.go:912` at pin N): Go slices the bytes from the
    /// cached position, so a cut char counts its bytes on both sides of the
    /// split. The text is sweepN2 case 11133 (`parserSkippedTokens16.ts`).
    /// Line 2 (0-based) starts at byte 36, and its `\u{AC}` is bytes 57 and
    /// 58. Go maps pos 57 to column 21 and pos 58 to column 22.
    #[test]
    fn line_character_cache_counts_cut_chars_as_go() {
        let text = "// @target: es2015\r\nfoo(): Bar { }\r\nfunction Foo      () \u{AC}   { }\r\n4+:5\r\nnamespace M {\r\nfunction a(\r\n    : T) { }\r\n}\r\nvar x       =";
        let source = SourceMapSource::Other(Rc::new(TextSource {
            line_map: compute_ecma_line_starts(text),
            text: text.to_string(),
        }));
        let mut cache = new_line_character_cache(&source);
        // Each call starts after the cached position: [57:58] counts 1 and
        // [58:61] counts 3 (the cut byte and 2 spaces).
        assert_eq!(cache.get_line_and_character(57), (2, 21));
        assert_eq!(cache.get_line_and_character(58), (2, 22));
        // The same position again: the empty range [58:58] counts 0.
        assert_eq!(cache.get_line_and_character(58), (2, 22));
        assert_eq!(cache.get_line_and_character(61), (2, 25));
        // A position before the cached one counts from the line start
        // ([36:56] counts 20). Then [56:61] has the whole char, which
        // counts 1.
        assert_eq!(cache.get_line_and_character(56), (2, 20));
        assert_eq!(cache.get_line_and_character(61), (2, 24));
        let mut fresh = new_line_character_cache(&source);
        assert_eq!(fresh.get_line_and_character(58), (2, 22));
    }
}
