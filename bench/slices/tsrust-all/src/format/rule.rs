use crate::format::prelude::*;

use crate::flags_macros::{go_enum, go_flags};
use std::sync::Arc;

// Go: format/rule.go:5 ruleImpl



// Go: format/rule.go:28 tokenRange
#[derive(Clone, Debug, Default)]
pub struct TokenRange {
    pub tokens: Vec<SyntaxKind>,
    pub is_specific: bool,
}

// Go: format/rule.go:33 ruleSpec


// Go: format/rule.go:71 toTokenRange
// PORT: the Go type switch is the `From` impls below. Go panics for any other
// argument type ("Unknown argument type passed to toTokenRange - only
// ast.Kind, []ast.Kind, and tokenRange supported"); in Rust that case does
// not compile.
pub fn to_token_range(e: impl Into<TokenRange>) -> TokenRange {
    e.into()
}

// Go: format/rule.go:73 `case ast.Kind`
impl From<SyntaxKind> for TokenRange {
    fn from(t: SyntaxKind) -> Self {
        TokenRange {
            is_specific: true,
            tokens: vec![t],
        }
    }
}

// Go: format/rule.go:75 `case []ast.Kind`
impl From<Vec<SyntaxKind>> for TokenRange {
    fn from(t: Vec<SyntaxKind>) -> Self {
        TokenRange {
            is_specific: true,
            tokens: t,
        }
    }
}

// Go: format/rule.go:77 `case tokenRange` is the identity `From<T> for T`.

// Go: format/rule.go:83 contextPredicate
// PORT: Go `func(ctx *FormattingContext) bool`. The rules map is a
// process-wide static, so predicates are `Send + Sync`. `&mut` because the
// FormattingContext getters cache tristates (Go mutates through the pointer).

// Go: format/rule.go:85 anyContext

// Go: format/rule.go:87 ruleAction
go_flags!(RuleAction, i32 {
    NONE = 0; // ruleActionNone
    STOP_PROCESSING_SPACE_ACTIONS = 1 << 0; // ruleActionStopProcessingSpaceActions
    STOP_PROCESSING_TOKEN_ACTIONS = 1 << 1; // ruleActionStopProcessingTokenActions
    INSERT_SPACE = 1 << 2; // ruleActionInsertSpace
    INSERT_NEW_LINE = 1 << 3; // ruleActionInsertNewLine
    DELETE_SPACE = 1 << 4; // ruleActionDeleteSpace
    DELETE_TOKEN = 1 << 5; // ruleActionDeleteToken
    INSERT_TRAILING_SEMICOLON = 1 << 6; // ruleActionInsertTrailingSemicolon

    STOP_ACTION = (1 << 0) | (1 << 1); // ruleActionStopProcessingSpaceActions | ruleActionStopProcessingTokenActions
    MODIFY_SPACE_ACTION = (1 << 2) | (1 << 3) | (1 << 4); // ruleActionInsertSpace | ruleActionInsertNewLine | ruleActionDeleteSpace
    MODIFY_TOKEN_ACTION = (1 << 5) | (1 << 6); // ruleActionDeleteToken | ruleActionInsertTrailingSemicolon
});

// Go: format/rule.go:104 ruleFlags
go_enum!(RuleFlags, i32 {
    NONE = 0; // ruleFlagsNone
    CAN_DELETE_NEW_LINES = 1; // ruleFlagsCanDeleteNewLines
});
