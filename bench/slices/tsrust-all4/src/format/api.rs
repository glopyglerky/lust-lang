use crate::format::prelude::*;

use crate::flags_macros::go_enum;
use crate::frontend::core_textchange::TextChange;
use crate::frontend::scanner::scanner_p1::{};
use crate::gostd::context::{ContextKey, with_value};
use std::sync::Arc;

// Go: format/api.go:14 FormatRequestKind
go_enum!(FormatRequestKind, i32 {
    FORMAT_DOCUMENT = 0; // FormatRequestKindFormatDocument
    FORMAT_SELECTION = 1; // FormatRequestKindFormatSelection
    FORMAT_ON_ENTER = 2; // FormatRequestKindFormatOnEnter
    FORMAT_ON_SEMICOLON = 3; // FormatRequestKindFormatOnSemicolon
    FORMAT_ON_OPENING_CURLY_BRACE = 4; // FormatRequestKindFormatOnOpeningCurlyBrace
    FORMAT_ON_CLOSING_CURLY_BRACE = 5; // FormatRequestKindFormatOnClosingCurlyBrace
});

// Go: format/api.go:25 formatContextKey
// PORT: Go keys both values with one int type (`formatOptionsKey`,
// `formatNewlineKey`). gostd context keys are typed, so each Go key value is
// its own static with the type of the value stored under it.

// Go: format/api.go:32 WithFormatCodeSettings

// Go: format/api.go:39 GetFormatCodeSettingsFromContext
// PORT: Go returns the settings by value; the context holds them in an Arc.

// Go: format/api.go:46 GetNewLineOrDefaultFromContext

// Go: format/api.go:58 FormatSpan

// Go: format/api.go:81 FormatNodeGivenIndentation

// Go: format/api.go:101 formatNodeLines

// Go: format/api.go:111 FormatDocument

// Go: format/api.go:115 FormatSelection

// Go: format/api.go:119 FormatOnOpeningCurly

// Go: format/api.go:142 FormatOnClosingCurly

// Go: format/api.go:147 FormatOnSemicolon

// Go: format/api.go:152 FormatOnEnter
