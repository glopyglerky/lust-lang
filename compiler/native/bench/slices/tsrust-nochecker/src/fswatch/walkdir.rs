//! Go: internal/fswatch/walkdir.go (the portable directory walk).
//!
//! PORT: Go uses it on platforms without a native walk (walkdir_other.go)
//! and in tests. The backends that walk use `walkdir_unix::walk_dir`: inotify
//! and fanotify on Linux, kqueue on darwin and the BSDs. Go's Windows walk
//! (walkdir_windows.go, FindFirstFile) is not ported: the Windows backend
//! does not walk, and only walkdir_test.go calls it there.
//! `os.Lstat` and `os.ReadDir` are `std::fs`, with Go's error texts
//! (`path_error`).

use crate::fswatch::prelude::*;

use crate::fswatch::syscall;
use crate::gostd::errors;

// Go: walkdir.go:14 walkDirGeneric

// Go: walkdir.go:25 walkDirGenericVisit

// PORT: Go `*fs.PathError` (`op path: err`) for a `std::io::Error`, with
// the Go text of its errno.
