//! Port of checker/printer.go: the checker entry points that print types,
//! symbols, signatures and type predicates through the node builder and the
//! printer.

use crate::prelude::*;
use crate::printer::{
    EmitTextWriter, PrinterOptions,
    get_single_line_string_writer, 
};
use std::cell::Cell;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};

/// Go `VerbosityContext` (checker/nodebuilder.go). Hover code sets it; the
/// checker passes nil.
// PORT: Go passes `*VerbosityContext`, and the node builder writes
// `CanIncreaseVerbosity` and `Truncated` back through the pointer. Here the
// two output fields are shared cells, so a clone that the node builder keeps
// writes to the same place as the caller's value.
#[derive(Clone, Debug, Default)]
pub struct VerbosityContext {
    pub level: i32,
    pub max_truncation_length: i32,
    pub can_increase_verbosity: Rc<Cell<bool>>,
    pub truncated: Rc<Cell<bool>>,
}




// Go: checker/printer.go:13 createPrinterWithDefaults

// Go: checker/printer.go:17 createPrinterWithRemoveComments

// Go: checker/printer.go:21 createPrinterWithRemoveCommentsOmitTrailingSemicolon

// Go: checker/printer.go:28 createPrinterWithRemoveCommentsOmitTrailingSemicolonNeverAsciiEscape

// Go: checker/printer.go:36 createPrinterWithRemoveCommentsNeverAsciiEscape

// Go: checker/printer.go:51 toNodeBuilderFlags
pub fn to_node_builder_flags(flags: TypeFormatFlags) -> NodeBuilderFlags {
    NodeBuilderFlags((flags & TypeFormatFlags::NODE_BUILDER_FLAGS_MASK).0)
}

// PORT: Go `ast.GetSourceFileOfNode(enclosingDeclaration)` guarded by a nil
// check appears in every entry point. `get_source_file_of_node` panics on
// nil, so the guard is kept at each call through this helper.

