//! Port of the parser fields of Go `ast.SourceFile` (`ast/ast.go:2467`).
//!
//! The Go parser returns a `*ast.SourceFile`, which is a node with extra
//! fields. Here the node lives in the node store of the file (`root`), and
//! the extra fields live in `ParsedSourceFile`. The parser (parser.go),
//! references.go and parseoptions.go set them.

use crate::frontend::prelude::*;






// PORT: Go `(*SourceFile).Diagnostics()` and `SetDiagnostics` read the
// node itself. Code that only holds the `SourceFile` node of a parsed store
// file (the tsconfig parser) reads the diagnostics from this table, keyed by
// store id. `parse_source_file` fills it.

// Go: ast.go (*SourceFile).SetDiagnostics

// Go: ast.go (*SourceFile).Diagnostics, for a parsed store file.
