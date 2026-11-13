//! Port of Go `ls/linkedediting.go`.

use crate::ls::prelude::*;

use crate::spanmap::Feature;

// Go: ls/linkedediting.go:16 jsxTagWordPattern
// allow the client to match more than valid tag names. This allows linked editing when typing is in progress or tag name is incomplete
// PORT: Go `*string` built with `new(...)`; the value never changes, so a
// `&'static str` holds it and each response copies it into `word_pattern`.
pub static JSX_TAG_WORD_PATTERN: &str = "[a-zA-Z0-9:\\-\\._$]*";

