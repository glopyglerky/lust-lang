//! Go `cmd/tsc/lsp.go`.

use crate::cmd::tsgo::prelude::*;

#[cfg(not(unix))]
use crate::cmd::tsgo::isprocessalive_other::{PROCESS_ALIVE_SUPPORTED, is_process_alive};
#[cfg(unix)]
use crate::cmd::tsgo::isprocessalive_unix::{PROCESS_ALIVE_SUPPORTED, };
use crate::cmd::tsgo::main::{ErrorHandling, new_flag_set, };
use crate::execute::tsc::compile::{};
use crate::execute::tsc::stdio;
use crate::frontend::bundled;
use crate::frontend::tspath;
use crate::frontend::vfs::osvfs;
use crate::gostd::context::{self, };
use crate::gostd::errors;
use std::io::Write;
use std::time::Duration;

// Go: cmd/tsc/lsp.go:20 runLSP

// Go: cmd/tsc/lsp.go:79 newParentProcessWatchdog
// newParentProcessWatchdog returns a SetParentProcessID callback if the platform
// supports process-alive checking and no client process ID override was provided,
// or nil otherwise.
// PORT: Go `clientProcessID` is an `int` (64 bits, see `FlagValue`); the
// watchdog takes the `i32` pid, as Linux `kill` takes a 32-bit `pid_t`.

// Go: cmd/tsc/lsp.go:95 startParentProcessWatchdog
// startParentProcessWatchdog starts a goroutine that monitors the parent process
// and cancels the context if the parent dies. This prevents orphaned language
// server processes when the editor crashes or is killed.

// Go: vfs/osvfs/os.go:193 GetGlobalTypingsCacheLocation
// PORT: ported here; `vfs/osvfs` is an accepted compiler file.

// Go: os/file.go:504 UserCacheDir
// PORT: the Windows, darwin and Unix branches (not plan9).
fn user_cache_dir() -> Result<String, GoError> {
    let getenv = |key: &str| std::env::var(key).unwrap_or_default();
    let mut dir;

    if cfg!(windows) {
        dir = getenv("LocalAppData");
        if dir.is_empty() {
            return Err(errors::new("%LocalAppData% is not defined"));
        }
    } else if cfg!(any(target_os = "macos", target_os = "ios")) {
        dir = getenv("HOME");
        if dir.is_empty() {
            return Err(errors::new("$HOME is not defined"));
        }
        dir.push_str("/Library/Caches");
    } else {
        // Unix
        dir = getenv("XDG_CACHE_HOME");
        if dir.is_empty() {
            dir = getenv("HOME");
            if dir.is_empty() {
                return Err(errors::new("neither $XDG_CACHE_HOME nor $HOME are defined"));
            }
            dir.push_str("/.cache");
        } else if !dir.starts_with('/') {
            // Go: !filepathlite.IsAbs(dir)
            return Err(errors::new("path in $XDG_CACHE_HOME is relative"));
        }
    }

    Ok(dir)
}

// Go: os/file_unix.go:390 tempDir
fn temp_dir() -> String {
    let mut dir = std::env::var("TMPDIR").unwrap_or_default();
    if dir.is_empty() {
        if cfg!(target_os = "android") {
            dir = "/data/local/tmp".to_string();
        } else {
            dir = "/tmp".to_string();
        }
    }
    dir
}
