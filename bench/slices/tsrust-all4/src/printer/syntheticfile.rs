//! Port of Go `printer/syntheticfile.go`.

use crate::prelude::*;

// Go: printer/syntheticfile.go:17 PrintAndPositionNode
// PrintAndPositionNode prints a synthesized node to text using the standard
// change-tracker printer options, trims the trailing newline, and assigns
// positions to the resulting node tree.
// sourceFile may be nil; when non-nil it is passed to the printer for comment
// preservation.
// The returned text is the printed source, and positioned is the node with
// concrete source positions assigned to it and all descendants.
// PORT: Go returns `(text string, positioned *ast.Node)` as a tuple. A nil
// Go `emitContext` is `None`. The printer writes through
// `Rc<RefCell<dyn EmitTextWriter>>`, so the writer is shared. The print
// handlers are taken first; they share the writer's positions (see
// `ChangeTrackerWriter`).

// Go: printer/syntheticfile.go:39 CreateSyntheticSourceFile
// CreateSyntheticSourceFile wraps a positioned node in a synthetic source file
// suitable for use with the formatter. The node must already have valid source
// positions assigned (e.g. via PrintAndPositionNode or AssignPositionsToNode).
// PORT: Go takes `parseOptions ast.SourceFileParseOptions`. The Rust
// `NodeFactory::new_source_file` keeps only its file name and path, so this
// takes those two. A factory SourceFile keeps `&'static` text, like the
// leaked synthetic nodes, so the text is leaked.
