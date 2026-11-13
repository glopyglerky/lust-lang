//! Go `internal/compiler/processingDiagnostic.go`: diagnostics found while
//! files are loaded. They become real diagnostics after the program is built.

use crate::frontend::prelude::*;

/// Go `processingDiagnosticKind`.
// PORT: Go `int` enum with iota constants, kept as a newtype like
// `FileIncludeKind`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ProcessingDiagnosticKind(pub i32);

impl ProcessingDiagnosticKind {
    pub const UNKNOWN_REFERENCE: ProcessingDiagnosticKind = ProcessingDiagnosticKind(0);
    pub const EXPLAINING_FILE_INCLUDE: ProcessingDiagnosticKind = ProcessingDiagnosticKind(1);
}




