//! Go scanner.go:2813-2931 comment range iteration.

use std::sync::LazyLock;

use crate::memchr::memmem;

use crate::frontend::prelude::*;
use crate::scanner_util::{is_shebang_trivia, };

use super::scanner_p1::{};

// Go: scanner/scanner.go:2799 GetLeadingCommentRanges
// PORT: Go returns an `iter.Seq`. All callers collect it, so this returns a Vec.

// Go: scanner/scanner.go:2803 GetTrailingCommentRanges

// Finds the "*/" that ends a block comment. PORT: Go uses strings.Index.
// One shared searcher gives the same first byte offset without the searcher
// setup that str::find and memmem::find do on each call.
static BLOCK_COMMENT_END: LazyLock<memmem::Finder<'static>> =
    LazyLock::new(|| memmem::Finder::new("*/"));

// Go: scanner/scanner.go:2813 iterateCommentRanges
