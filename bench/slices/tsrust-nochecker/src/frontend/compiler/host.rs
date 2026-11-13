//! Go `internal/compiler/host.go`: the compiler host that reads and parses
//! source files and resolves project references.

use crate::contentmapper::{self, Mapper, };
use crate::frontend::prelude::*;
use crate::gostd::GoError;





// Go: host.go:44 NewCachedFSCompilerHost


// Go: host.go:55 NewCompilerHost



// PORT: Go passes the `*compilerHost` as a `tsoptions.ParseConfigHost`
// (it has `FS()` and `GetCurrentDirectory()`). Rust needs the explicit impl.
