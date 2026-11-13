//! Go `internal/compiler/includeprocessor.go`: include reasons per file,
//! the processing diagnostics, and caches used to explain why a file is in
//! the program.

use crate::frontend::prelude::*;
use std::cell::OnceCell;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};


// PORT: Go `UpdateProgram` copies `processedFiles` and so shares the
// `*includeProcessor`. `updateFileIncludeProcessor` replaces it right after,
// so a clone only needs the reasons and the diagnostics. The caches start
// empty.

// Go: includeprocessor.go:28 updateFileIncludeProcessor
// PORT: Go replaces the processor of `p` in place and shares the reason map
// and the diagnostics slice. Rust shares the reason map and clones the
// diagnostics (the old program keeps its own).

