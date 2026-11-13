//! Go `scanner` entry points for the language service
//! (`scanner/scanner.go:2519-2527` `GetScannerForSourceFile`,
//! `scanner/scanner.go:2738-2740` `GetECMAPositionOfLineAndByteOffset`).
//!
//! The compiler's `scanner_util` wrappers (`ScanTokenAtPosition`,
//! `GetRangeOfTokenAtPosition`, `GetErrorRangeForNode`) also use this
//! function.

use crate::frontend::prelude::*;
use crate::frontend::scanner::{};

// Go: scanner/scanner.go:2505 GetScannerForSourceFile
// PORT: the scanner borrows the text of `source_file`, so the caller holds
// it (`let text = source_file_text(source_file);`) while it scans.

// Go: scanner/scanner.go:2724 GetECMAPositionOfLineAndByteOffset
