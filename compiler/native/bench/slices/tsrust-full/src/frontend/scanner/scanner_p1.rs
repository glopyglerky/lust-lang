//! Port of `scanner/scanner.go` lines 1 to 1236: the `Scanner` struct and
//! state, `Scan`, comment directives and the rescans for `<`, `>`, `/`, `*=`
//! and templates.
//!
//! Go `rune` ports to `i32` (Go `char()` returns -1 at the end of the text).
//! The Go `stringutil` predicates take `char`; `rune_to_char` maps a rune to
//! a `char` (invalid values become U+FFFD, which none of them accept).
//!
//! Go indexes strings by byte. This port reads `text.as_bytes()` and decodes
//! runes with `utf8_decode_rune_in_string`, which follows Go
//! `utf8.DecodeRuneInString` also at a non-boundary position.

use crate::flags_macros::{go_enum, go_flags};
use crate::frontend::prelude::*;

use super::regexp::{RegularExpressionFlags, };

// Go: scanner/scanner.go:21 identifierVariant (ts#63996)
go_enum!(IdentifierVariant, i32 {
    STANDARD = 0; // identifierVariantStandard
    JSX = 1; // identifierVariantJSX
    REG_EXP_GROUP_NAME = 2; // identifierVariantRegExpGroupName
});

go_flags!(EscapeSequenceScanningFlags, i32 {
    STRING = 1 << 0; // EscapeSequenceScanningFlagsString
    REPORT_ERRORS = 1 << 1; // EscapeSequenceScanningFlagsReportErrors
    REGULAR_EXPRESSION = 1 << 2; // EscapeSequenceScanningFlagsRegularExpression
    ANNEX_B = 1 << 3; // EscapeSequenceScanningFlagsAnnexB
    ANY_UNICODE_MODE = 1 << 4; // EscapeSequenceScanningFlagsAnyUnicodeMode
    ATOM_ESCAPE = 1 << 5; // EscapeSequenceScanningFlagsAtomEscape
    REPORT_INVALID_ESCAPE_ERRORS = (1 << 2) | (1 << 1); // EscapeSequenceScanningFlagsReportInvalidEscapeErrors
    ALLOW_EXTENDED_UNICODE_ESCAPE = (1 << 0) | (1 << 4); // EscapeSequenceScanningFlagsAllowExtendedUnicodeEscape
});


// ---------------------------------------------------------------------------
// Go `unicode/utf8` helpers.
// ---------------------------------------------------------------------------

/// Go `utf8.RuneError`.
pub(crate) const RUNE_ERROR: i32 = 0xFFFD;
/// Go `utf8.RuneSelf`.
pub(crate) const RUNE_SELF: i32 = 0x80;



/// First UTF-8 byte of `GO_STRING_MARKER` (U+FDD0 is EF B7 90).
const GO_STRING_MARKER_LEAD: u8 = 0xEF;

/// The Go rune of a port form unit and its size in the text (see
/// `utf8_decode_rune_in_string`).
fn go_unit_rune((unit, size): (GoUnit, usize)) -> (i32, i32) {
    match unit {
        GoUnit::Char(ch) => (ch as i32, size as i32),
        GoUnit::Surrogate(_) | GoUnit::InvalidByte(_) => (RUNE_ERROR, size as i32),
    }
}

/// Go `utf8.DecodeRune(b)` on a byte slice.
fn decode_rune_in_bytes(bytes: &[u8]) -> (i32, i32) {
    if bytes.is_empty() {
        return (RUNE_ERROR, 0);
    }
    let b = bytes[0];
    if i32::from(b) < RUNE_SELF {
        return (i32::from(b), 1);
    }
    let width = if b & 0xE0 == 0xC0 {
        2
    } else if b & 0xF0 == 0xE0 {
        3
    } else if b & 0xF8 == 0xF0 {
        4
    } else {
        return (RUNE_ERROR, 1);
    };
    if width > bytes.len() {
        return (RUNE_ERROR, 1);
    }
    match std::str::from_utf8(&bytes[..width])
        .ok()
        .and_then(|s| s.chars().next())
    {
        Some(ch) => (ch as i32, width as i32),
        None => (RUNE_ERROR, 1),
    }
}






// ---------------------------------------------------------------------------
// Keyword and token tables.
// ---------------------------------------------------------------------------

/// Go `textToKeyword` map, in Go source order.
pub(crate) static TEXT_TO_KEYWORD: &[(&str, SyntaxKind)] = &[
    ("abstract", SyntaxKind::AbstractKeyword),
    ("accessor", SyntaxKind::AccessorKeyword),
    ("any", SyntaxKind::AnyKeyword),
    ("as", SyntaxKind::AsKeyword),
    ("asserts", SyntaxKind::AssertsKeyword),
    ("assert", SyntaxKind::AssertKeyword),
    ("bigint", SyntaxKind::BigIntKeyword),
    ("boolean", SyntaxKind::BooleanKeyword),
    ("break", SyntaxKind::BreakKeyword),
    ("case", SyntaxKind::CaseKeyword),
    ("catch", SyntaxKind::CatchKeyword),
    ("class", SyntaxKind::ClassKeyword),
    ("continue", SyntaxKind::ContinueKeyword),
    ("const", SyntaxKind::ConstKeyword),
    ("constructor", SyntaxKind::ConstructorKeyword),
    ("debugger", SyntaxKind::DebuggerKeyword),
    ("declare", SyntaxKind::DeclareKeyword),
    ("default", SyntaxKind::DefaultKeyword),
    ("defer", SyntaxKind::DeferKeyword),
    ("delete", SyntaxKind::DeleteKeyword),
    ("do", SyntaxKind::DoKeyword),
    ("else", SyntaxKind::ElseKeyword),
    ("enum", SyntaxKind::EnumKeyword),
    ("export", SyntaxKind::ExportKeyword),
    ("extends", SyntaxKind::ExtendsKeyword),
    ("false", SyntaxKind::FalseKeyword),
    ("finally", SyntaxKind::FinallyKeyword),
    ("for", SyntaxKind::ForKeyword),
    ("from", SyntaxKind::FromKeyword),
    ("function", SyntaxKind::FunctionKeyword),
    ("get", SyntaxKind::GetKeyword),
    ("if", SyntaxKind::IfKeyword),
    ("immediate", SyntaxKind::ImmediateKeyword),
    ("implements", SyntaxKind::ImplementsKeyword),
    ("import", SyntaxKind::ImportKeyword),
    ("in", SyntaxKind::InKeyword),
    ("infer", SyntaxKind::InferKeyword),
    ("instanceof", SyntaxKind::InstanceOfKeyword),
    ("interface", SyntaxKind::InterfaceKeyword),
    ("intrinsic", SyntaxKind::IntrinsicKeyword),
    ("is", SyntaxKind::IsKeyword),
    ("keyof", SyntaxKind::KeyOfKeyword),
    ("let", SyntaxKind::LetKeyword),
    ("module", SyntaxKind::ModuleKeyword),
    ("namespace", SyntaxKind::NamespaceKeyword),
    ("never", SyntaxKind::NeverKeyword),
    ("new", SyntaxKind::NewKeyword),
    ("null", SyntaxKind::NullKeyword),
    ("number", SyntaxKind::NumberKeyword),
    ("object", SyntaxKind::ObjectKeyword),
    ("package", SyntaxKind::PackageKeyword),
    ("private", SyntaxKind::PrivateKeyword),
    ("protected", SyntaxKind::ProtectedKeyword),
    ("public", SyntaxKind::PublicKeyword),
    ("override", SyntaxKind::OverrideKeyword),
    ("out", SyntaxKind::OutKeyword),
    ("readonly", SyntaxKind::ReadonlyKeyword),
    ("require", SyntaxKind::RequireKeyword),
    ("global", SyntaxKind::GlobalKeyword),
    ("return", SyntaxKind::ReturnKeyword),
    ("satisfies", SyntaxKind::SatisfiesKeyword),
    ("set", SyntaxKind::SetKeyword),
    ("static", SyntaxKind::StaticKeyword),
    ("string", SyntaxKind::StringKeyword),
    ("super", SyntaxKind::SuperKeyword),
    ("switch", SyntaxKind::SwitchKeyword),
    ("symbol", SyntaxKind::SymbolKeyword),
    ("this", SyntaxKind::ThisKeyword),
    ("throw", SyntaxKind::ThrowKeyword),
    ("true", SyntaxKind::TrueKeyword),
    ("try", SyntaxKind::TryKeyword),
    ("type", SyntaxKind::TypeKeyword),
    ("typeof", SyntaxKind::TypeOfKeyword),
    ("undefined", SyntaxKind::UndefinedKeyword),
    ("unique", SyntaxKind::UniqueKeyword),
    ("unknown", SyntaxKind::UnknownKeyword),
    ("using", SyntaxKind::UsingKeyword),
    ("var", SyntaxKind::VarKeyword),
    ("void", SyntaxKind::VoidKeyword),
    ("while", SyntaxKind::WhileKeyword),
    ("with", SyntaxKind::WithKeyword),
    ("yield", SyntaxKind::YieldKeyword),
    ("async", SyntaxKind::AsyncKeyword),
    ("await", SyntaxKind::AwaitKeyword),
    ("of", SyntaxKind::OfKeyword),
];

/// Go `textToToken` punctuation entries. Go copies `textToKeyword` into the
/// same map; `text_to_token` checks both tables.
pub(crate) static TEXT_TO_PUNCTUATION: &[(&str, SyntaxKind)] = &[
    ("{", SyntaxKind::OpenBraceToken),
    ("}", SyntaxKind::CloseBraceToken),
    ("(", SyntaxKind::OpenParenToken),
    (")", SyntaxKind::CloseParenToken),
    ("[", SyntaxKind::OpenBracketToken),
    ("]", SyntaxKind::CloseBracketToken),
    (".", SyntaxKind::DotToken),
    ("...", SyntaxKind::DotDotDotToken),
    (";", SyntaxKind::SemicolonToken),
    (",", SyntaxKind::CommaToken),
    ("<", SyntaxKind::LessThanToken),
    (">", SyntaxKind::GreaterThanToken),
    ("<=", SyntaxKind::LessThanEqualsToken),
    (">=", SyntaxKind::GreaterThanEqualsToken),
    ("==", SyntaxKind::EqualsEqualsToken),
    ("!=", SyntaxKind::ExclamationEqualsToken),
    ("===", SyntaxKind::EqualsEqualsEqualsToken),
    ("!==", SyntaxKind::ExclamationEqualsEqualsToken),
    ("=>", SyntaxKind::EqualsGreaterThanToken),
    ("+", SyntaxKind::PlusToken),
    ("-", SyntaxKind::MinusToken),
    ("**", SyntaxKind::AsteriskAsteriskToken),
    ("*", SyntaxKind::AsteriskToken),
    ("/", SyntaxKind::SlashToken),
    ("%", SyntaxKind::PercentToken),
    ("++", SyntaxKind::PlusPlusToken),
    ("--", SyntaxKind::MinusMinusToken),
    ("<<", SyntaxKind::LessThanLessThanToken),
    ("</", SyntaxKind::LessThanSlashToken),
    (">>", SyntaxKind::GreaterThanGreaterThanToken),
    (">>>", SyntaxKind::GreaterThanGreaterThanGreaterThanToken),
    ("&", SyntaxKind::AmpersandToken),
    ("|", SyntaxKind::BarToken),
    ("^", SyntaxKind::CaretToken),
    ("!", SyntaxKind::ExclamationToken),
    ("~", SyntaxKind::TildeToken),
    ("&&", SyntaxKind::AmpersandAmpersandToken),
    ("||", SyntaxKind::BarBarToken),
    ("?", SyntaxKind::QuestionToken),
    ("??", SyntaxKind::QuestionQuestionToken),
    ("?.", SyntaxKind::QuestionDotToken),
    (":", SyntaxKind::ColonToken),
    ("=", SyntaxKind::EqualsToken),
    ("+=", SyntaxKind::PlusEqualsToken),
    ("-=", SyntaxKind::MinusEqualsToken),
    ("*=", SyntaxKind::AsteriskEqualsToken),
    ("**=", SyntaxKind::AsteriskAsteriskEqualsToken),
    ("/=", SyntaxKind::SlashEqualsToken),
    ("%=", SyntaxKind::PercentEqualsToken),
    ("<<=", SyntaxKind::LessThanLessThanEqualsToken),
    (">>=", SyntaxKind::GreaterThanGreaterThanEqualsToken),
    (
        ">>>=",
        SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken,
    ),
    ("&=", SyntaxKind::AmpersandEqualsToken),
    ("|=", SyntaxKind::BarEqualsToken),
    ("^=", SyntaxKind::CaretEqualsToken),
    ("||=", SyntaxKind::BarBarEqualsToken),
    ("&&=", SyntaxKind::AmpersandAmpersandEqualsToken),
    ("??=", SyntaxKind::QuestionQuestionEqualsToken),
    ("@", SyntaxKind::AtToken),
    ("#", SyntaxKind::HashToken),
    ("`", SyntaxKind::BacktickToken),
];

/// Go `textToKeyword[text]`. Returns `SyntaxKind::Unknown` (the Go zero
/// value) on a miss.
// PERF: a match on the length, then on the bytes, so a lookup does not hash
// the text. It holds the same entries as `TEXT_TO_KEYWORD`
// (`tests::text_to_keyword_matches_table` checks this). `crate::scanner_util`
// uses it too.
pub(crate) fn text_to_keyword(text: &str) -> SyntaxKind {
    use crate::astdata::SyntaxKind as K;
    let b = text.as_bytes();
    match b.len() {
        2 => match b {
            b"as" => K::AsKeyword,
            b"do" => K::DoKeyword,
            b"if" => K::IfKeyword,
            b"in" => K::InKeyword,
            b"is" => K::IsKeyword,
            b"of" => K::OfKeyword,
            _ => K::Unknown,
        },
        3 => match b {
            b"any" => K::AnyKeyword,
            b"for" => K::ForKeyword,
            b"get" => K::GetKeyword,
            b"let" => K::LetKeyword,
            b"new" => K::NewKeyword,
            b"out" => K::OutKeyword,
            b"set" => K::SetKeyword,
            b"try" => K::TryKeyword,
            b"var" => K::VarKeyword,
            _ => K::Unknown,
        },
        4 => match b {
            b"case" => K::CaseKeyword,
            b"else" => K::ElseKeyword,
            b"enum" => K::EnumKeyword,
            b"from" => K::FromKeyword,
            b"null" => K::NullKeyword,
            b"this" => K::ThisKeyword,
            b"true" => K::TrueKeyword,
            b"type" => K::TypeKeyword,
            b"void" => K::VoidKeyword,
            b"with" => K::WithKeyword,
            _ => K::Unknown,
        },
        5 => match b {
            b"async" => K::AsyncKeyword,
            b"await" => K::AwaitKeyword,
            b"break" => K::BreakKeyword,
            b"catch" => K::CatchKeyword,
            b"class" => K::ClassKeyword,
            b"const" => K::ConstKeyword,
            b"defer" => K::DeferKeyword,
            b"false" => K::FalseKeyword,
            b"infer" => K::InferKeyword,
            b"keyof" => K::KeyOfKeyword,
            b"never" => K::NeverKeyword,
            b"super" => K::SuperKeyword,
            b"throw" => K::ThrowKeyword,
            b"using" => K::UsingKeyword,
            b"while" => K::WhileKeyword,
            b"yield" => K::YieldKeyword,
            _ => K::Unknown,
        },
        6 => match b {
            b"assert" => K::AssertKeyword,
            b"bigint" => K::BigIntKeyword,
            b"delete" => K::DeleteKeyword,
            b"export" => K::ExportKeyword,
            b"global" => K::GlobalKeyword,
            b"import" => K::ImportKeyword,
            b"module" => K::ModuleKeyword,
            b"number" => K::NumberKeyword,
            b"object" => K::ObjectKeyword,
            b"public" => K::PublicKeyword,
            b"return" => K::ReturnKeyword,
            b"static" => K::StaticKeyword,
            b"string" => K::StringKeyword,
            b"switch" => K::SwitchKeyword,
            b"symbol" => K::SymbolKeyword,
            b"typeof" => K::TypeOfKeyword,
            b"unique" => K::UniqueKeyword,
            _ => K::Unknown,
        },
        7 => match b {
            b"asserts" => K::AssertsKeyword,
            b"boolean" => K::BooleanKeyword,
            b"declare" => K::DeclareKeyword,
            b"default" => K::DefaultKeyword,
            b"extends" => K::ExtendsKeyword,
            b"finally" => K::FinallyKeyword,
            b"package" => K::PackageKeyword,
            b"private" => K::PrivateKeyword,
            b"require" => K::RequireKeyword,
            b"unknown" => K::UnknownKeyword,
            _ => K::Unknown,
        },
        8 => match b {
            b"abstract" => K::AbstractKeyword,
            b"accessor" => K::AccessorKeyword,
            b"continue" => K::ContinueKeyword,
            b"debugger" => K::DebuggerKeyword,
            b"function" => K::FunctionKeyword,
            b"override" => K::OverrideKeyword,
            b"readonly" => K::ReadonlyKeyword,
            _ => K::Unknown,
        },
        9 => match b {
            b"immediate" => K::ImmediateKeyword,
            b"interface" => K::InterfaceKeyword,
            b"intrinsic" => K::IntrinsicKeyword,
            b"namespace" => K::NamespaceKeyword,
            b"protected" => K::ProtectedKeyword,
            b"satisfies" => K::SatisfiesKeyword,
            b"undefined" => K::UndefinedKeyword,
            _ => K::Unknown,
        },
        10 => match b {
            b"implements" => K::ImplementsKeyword,
            b"instanceof" => K::InstanceOfKeyword,
            _ => K::Unknown,
        },
        11 => match b {
            b"constructor" => K::ConstructorKeyword,
            _ => K::Unknown,
        },
        _ => K::Unknown,
    }
}

/// Go `kind, ok := textToToken[text]`.
pub(crate) fn text_to_token(text: &str) -> Option<SyntaxKind> {
    static MAP: std::sync::OnceLock<FxHashMap<&'static str, SyntaxKind>> =
        std::sync::OnceLock::new();
    MAP.get_or_init(|| {
        let mut m: FxHashMap<&'static str, SyntaxKind> =
            TEXT_TO_PUNCTUATION.iter().copied().collect();
        // Go: maps.Copy(m, textToKeyword)
        m.extend(TEXT_TO_KEYWORD.iter().copied());
        m
    })
    .get(text)
    .copied()
}

// ---------------------------------------------------------------------------
// Scanner
// ---------------------------------------------------------------------------

/// Go `scanner.ScannerState`.
// PORT: `commentDirectives` is a Go slice header. Go `Rewind` restores its
// length, and later appends overwrite the shared backing array after that
// length. Here the backing array is `Scanner.comment_directives` and the
// state keeps only the length (`comment_directives_len`). `tokenValue` is a
// slice of the scanned text or an interned `&'static str` (see
// `intern_token_value`), so the state is `Copy`.
#[derive(Clone, Copy, Debug)]
pub struct ScannerState<'a> {
    /// Current position in text (and ending position of current token)
    pub pos: i32,
    /// Starting position of current token including preceding whitespace
    pub full_start_pos: i32,
    /// Starting position of non-whitespace part of current token
    pub token_start: i32,
    /// Kind of current token
    pub token: SyntaxKind,
    /// Parsed value of current token
    pub token_value: &'a str,
    /// Flags for current token
    pub token_flags: TokenFlags,
    pub comment_directives_len: usize,
    /// Leading asterisks to skip when scanning types inside JSDoc. Should be 0 outside JSDoc
    pub skip_js_doc_leading_asterisks: i32,
}

impl Default for ScannerState<'_> {
    fn default() -> Self {
        ScannerState {
            pos: 0,
            full_start_pos: 0,
            token_start: 0,
            token: SyntaxKind::Unknown,
            token_value: "",
            token_flags: TokenFlags::NONE,
            comment_directives_len: 0,
            skip_js_doc_leading_asterisks: 0,
        }
    }
}


// Go: scanner/scanner.go:225 defaultScanner

// Go: scanner/scanner.go:232 NewScanner


// Go: scanner/scanner.go:247 cleared
pub(crate) fn cleared<K, V>(mut m: FxHashMap<K, V>) -> FxHashMap<K, V> {
    m.clear();
    m
}


// Go: scanner/scanner.go:380 hasJSDocTag
// hasJSDocTag reports whether text starts with one of the given tag names followed
// by a valid JSDoc tag terminator (whitespace, '}', '*', or end-of-string).
pub(crate) fn has_js_doc_tag(text: &str, tags: &[&str]) -> bool {
    for tag in tags {
        if !text.starts_with(tag) {
            continue;
        }
        if text.len() == tag.len() {
            return true;
        }
        let ch = text.as_bytes()[tag.len()];
        if ch == b' ' || ch == b'\t' || ch == b'\n' || ch == b'\r' || ch == b'}' || ch == b'*' {
            return true;
        }
    }
    false
}


#[cfg(test)]
mod tests {
    use super::*;

    /// Go `textToKeyword[text]` read from the table.
    fn table_keyword(text: &str) -> SyntaxKind {
        TEXT_TO_KEYWORD
            .iter()
            .find(|&&(t, _)| t == text)
            .map_or(SyntaxKind::Unknown, |&(_, kind)| kind)
    }

    // `text_to_keyword` is a hand-written match. Every keyword and some near
    // misses (longer, shorter, other case, other last byte) must give the
    // table result.
    #[test]
    fn text_to_keyword_matches_table() {
        for &(text, kind) in TEXT_TO_KEYWORD {
            assert_eq!(text_to_keyword(text), kind, "{text}");
            let last = text.len() - 1;
            let near = [
                format!("{text}s"),
                text[..last].to_string(),
                text.to_uppercase(),
                format!("{}z", &text[..last]),
            ];
            for miss in &near {
                assert_eq!(text_to_keyword(miss), table_keyword(miss), "{miss}");
            }
        }
        assert_eq!(text_to_keyword(""), SyntaxKind::Unknown);
        assert_eq!(text_to_keyword("{"), SyntaxKind::Unknown);
    }
}
