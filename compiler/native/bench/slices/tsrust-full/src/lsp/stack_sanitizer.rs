//! Go `internal/lsp/stack_sanitizer.go`.

use crate::lsp::prelude::*;

// VS Code's telemetry pipeline redacts any string matching
// /(key|token|sig|secret|signature|password|passwd|pwd|android:value)[^a-zA-Z0-9]/i
// as `<REDACTED: Generic Secret>`, which trips on innocuous Go frames like
// `getSignatureHelp(`. Insert `X_X` after each trigger keyword that we know
// can appear in our sanitized output, when followed by punctuation we
// actually emit (`(`, `[`, `.`, `|`); reverse by removing the marker (replace
// `X_X` with the empty string) on the dashboard.
//
// Go: lsp/stack_sanitizer.go:17 genericSecretKeywordRegex
// PORT: no regex engine. The Go pattern is
// `(?i)(key|token|signature|sig|pwd)([(\[.|])`; `GENERIC_SECRET_KEYWORDS`
// holds the alternatives in pattern order and `defeat_generic_secret_regex`
// scans for them the way Go's leftmost-first matcher does.
pub const GENERIC_SECRET_KEYWORDS: [&str; 5] = ["key", "token", "signature", "sig", "pwd"];

// Go: `[(\[.|]`, the second group of genericSecretKeywordRegex.
fn is_generic_secret_punctuation(c: char) -> bool {
    matches!(c, '(' | '[' | '.' | '|')
}

// Go `(?i)` matching of one pattern letter (always lowercase ASCII here)
// against one rune: the simple case-folding orbit of the letter. Among the
// letters of the keywords only `k` (K, U+212A KELVIN SIGN) and `s` (S,
// U+017F LATIN SMALL LETTER LONG S) have a non-ASCII member.
fn fold_eq(pattern: char, c: char) -> bool {
    if c.to_ascii_lowercase() == pattern {
        return true;
    }
    match pattern {
        'k' => c == '\u{212A}',
        's' => c == '\u{017F}',
        _ => false,
    }
}

// Tries the pattern at byte offset `start`: the first keyword (in pattern
// order) that matches, followed by one punctuation rune. Returns the end of
// the keyword and the end of the whole match.

// Go: lsp/stack_sanitizer.go:19 defeatGenericSecretRegex
// Go: `genericSecretKeywordRegex.ReplaceAllString(s, "${1}X_X${2}")`.

// Go: lsp/stack_sanitizer.go:23 sanitizeStackTrace

// Go: lsp/stack_sanitizer.go:64 writeSanitizedModuleOrPath
