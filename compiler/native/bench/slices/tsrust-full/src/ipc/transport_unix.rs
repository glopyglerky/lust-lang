//! Port of internal/ipc/transport_unix.go (`//go:build !windows`;
//! internal/api/transport_unix.go before tsgo#4712).
//!
//! PORT: Go `net.Listen("unix", path)` and `net.UnixConn` are
//! `std::os::unix::net`. Errors are Go's `*net.OpError` texts ("op unix
//! addr: syscall: errno text").

use crate::ipc::prelude::*;

use crate::gostd::{GoError, errors};
use crate::ipc::transport::{ReadWriteCloser};
use std::net::Shutdown;
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, Mutex};

// Go: ipc/transport_unix.go:12 newPipeListener
// newPipeListener creates a Unix domain socket listener.


// Go: ipc/transport_unix.go:19 GeneratePipePath
// GeneratePipePath returns a platform-appropriate pipe path for the given name.

// Go `os.TempDir` on Unix: $TMPDIR, or "/tmp" when it is empty.
fn os_temp_dir() -> String {
    match std::env::var("TMPDIR") {
        Ok(dir) if !dir.is_empty() => dir,
        _ => "/tmp".to_string(),
    }
}

// Go `path.Join(dir, name)`: the non-empty elements joined by "/", then
// cleaned.

// Go: internal/poll/sock_cloexec.go (accept4) and sys_cloexec.go (accept),
// the `errcall` of an accept error.
#[cfg(not(target_vendor = "apple"))]
const ACCEPT_CALL: &str = "accept4";
#[cfg(target_vendor = "apple")]
const ACCEPT_CALL: &str = "accept";




