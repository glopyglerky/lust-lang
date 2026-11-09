use crate::format::prelude::*;

use crate::flags_macros::go_enum;
use crate::frontend::core_textchange::TextChange;
use crate::frontend::scanner::{};

// Go: format/scanner.go:12 TextRangeWithKind
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextRangeWithKind {
    pub loc: TextRange,
    pub kind: SyntaxKind,
}

// PORT: the Go zero value (Loc 0..0, KindUnknown). crate::astdata::SyntaxKind has no
// Default.
impl Default for TextRangeWithKind {
    fn default() -> Self {
        new_text_range_with_kind(0, 0, SyntaxKind::Unknown)
    }
}

// Go: format/scanner.go:17 NewTextRangeWithKind
pub fn new_text_range_with_kind(pos: i32, end: i32, kind: SyntaxKind) -> TextRangeWithKind {
    TextRangeWithKind {
        loc: TextRange::new(pos, end),
        kind,
    }
}

// Go: format/scanner.go:24 tokenInfo
#[derive(Clone, Debug, Default)]
pub struct TokenInfo {
    pub leading_trivia: Vec<TextRangeWithKind>,
    pub token: TextRangeWithKind,
    pub trailing_trivia: Vec<TextRangeWithKind>,
}

// Go: format/scanner.go:30 formattingScanner
// PORT: Go `s *scanner.Scanner` is owned here (the literal
// frontend::scanner::Scanner port, which has the JSX rescans).

// Go: format/scanner.go:43 newFormattingScanner
// PORT: Go passes `*formatSpanWorker` and the worker keeps a pointer to the
// formatting scanner. Here the worker is moved in, `execute` moves the
// formatting scanner into the worker, and the worker's edits are returned.


// Go: format/scanner.go:99 shouldRescanGreaterThanToken

// Go: format/scanner.go:111 shouldRescanJsxIdentifier

// Go: format/scanner.go:129 isLeftmostJsxTagName


// Go: format/scanner.go:155 shouldRescanSlashToken

// Go: format/scanner.go:159 shouldRescanTemplateToken

// Go: format/scanner.go:164 shouldRescanJsxAttributeValue

// Go: format/scanner.go:168 startsWithSlashToken
pub fn starts_with_slash_token(t: SyntaxKind) -> bool {
    t == SyntaxKind::SlashToken || t == SyntaxKind::SlashEqualsToken
}

// Go: format/scanner.go:172 scanAction
go_enum!(ScanAction, i32 {
    ACTION_SCAN = 0; // actionScan
    ACTION_RESCAN_GREATER_THAN_TOKEN = 1; // actionRescanGreaterThanToken
    ACTION_RESCAN_SLASH_TOKEN = 2; // actionRescanSlashToken
    ACTION_RESCAN_TEMPLATE_TOKEN = 3; // actionRescanTemplateToken
    ACTION_RESCAN_JSX_IDENTIFIER = 4; // actionRescanJsxIdentifier
    ACTION_RESCAN_JSX_TEXT = 5; // actionRescanJsxText
    ACTION_RESCAN_JSX_ATTRIBUTE_VALUE = 6; // actionRescanJsxAttributeValue
});

// Go: format/scanner.go:184 fixTokenKind

