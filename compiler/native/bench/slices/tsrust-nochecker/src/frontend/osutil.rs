//! Go: internal/osutil (osutil.go and os_other.go), tsgo#4734.
//!
//! PORT: the port builds for Linux only (as `nativepath`), so os_android.go
//! (the Termux launcher case) is not ported. Go `(string, error)` is
//! `io::Result<String>`. Strings are in the port form (see
//! `vfs::osvfs::os_path`).

use std::io;

// Go: osutil.go:4 Args
// Args returns the command-line arguments with platform-specific launcher details removed.

// Go: osutil.go:9 Executable
// Executable returns the path of the current executable, accounting for platform-specific launchers.

/// Go os_other.go (`//go:build !android`).
mod os_other {
    use std::io;

    // Go: os_other.go:7 args

    // Go: os_other.go:11 executable
}
