//! Port of Go `ls/diagnostics.go`.

use crate::ls::prelude::*;

use crate::spanmap::{Fidelity, SpanMap};

// Go: ls/diagnostics.go:18 getAllDiagnostics
// getAllDiagnostics collects all diagnostics for a file: syntactic, semantic,
// suggestion, and (when declarations are emitted) declaration diagnostics.
// PORT: Go `*compiler.Program` methods are the `ls_program` functions
// (plan contract C3).


// Go: ls/diagnostics.go:84 isSynthesizedContentMappedDiagnostic
// isSynthesizedContentMappedDiagnostic reports whether diag is a compiler diagnostic on a content-mapped
// file whose location lies entirely in synthesized virtual code with no counterpart in the original
// file, and so has no meaningful position to report against the original file.

// Go: ls/diagnostics.go:97 aggregateSynthesizedDiagnostics
// aggregateSynthesizedDiagnostics builds a single diagnostic at the top of a content-mapped file standing
// in for compiler diagnostics located in synthesized code with no original location. The originals are
// attached as related information so their messages are surfaced rather than silently dropped. (A later
// change will point the related locations at a read-only view of the file's virtual TypeScript.)

// Go: ls/diagnostics.go:109 worstCategory
