//! Go `os.Stdin`, `os.Stdout` and `os.Stderr`: os/file.go `File.Read` and
//! `File.Write`, os/file_unix.go `NewFile` and `epipecheck`,
//! internal/poll/fd_unix.go `FD.Read` and `FD.Write`.
//!
//! Each read or write of these files:
//! - tries again after EINTR (internal/poll `ignoringEINTRIO`);
//! - waits in the poller on EAGAIN, so a non-blocking fd (a Node or libuv
//!   parent can set O_NONBLOCK on a shared pipe or tty) loses no output and
//!   a read does not fail;
//! - on EPIPE from a write to fd 1 or 2, ends the process by SIGPIPE with
//!   the default action (`epipecheck`, runtime/signal_unix.go `sigpipe` and
//!   `dieFromSignal`). Go does this also when SIGPIPE was ignored at start.
//!
//! PORT: Go waits on EAGAIN only for an fd that was non-blocking at start
//! (`NewFile` gives only such an fd to the poller). Here every EAGAIN waits,
//! also when a parent sets O_NONBLOCK later; Go then returns the EAGAIN
//! error, which `fmt.Fprint` ignores (`CliStdout` keeps the Go rule). Rust
//! ignores SIGPIPE, so a write here gets EPIPE and raises the signal as Go
//! does. A write that writes 0 bytes gives `WriteZero` (Go
//! `io.ErrUnexpectedEOF`). As in Go, none of these files has a buffer,
//! except `CliStdout` on a regular file inside a report (`keep_writes`). On
//! Windows these are the std handles, as before.

use std::cell::Cell;
use std::io;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

/// Go `os.Stdin`. Wrap it in a `BufReader` where Go wraps it in a
/// `bufio.Reader`.
pub struct Stdin;

/// Go `os.Stdout`, not buffered. The LSP and API servers use it under the
/// `bufio.Writer` of their base protocol.
pub struct Stdout;

/// Go `os.Stdout` as the tsc system writer uses it: each write is one
/// write of fd 1 (Go `fmt.Fprint` on the unbuffered `os.Stdout`; an empty
/// one too), so the output goes out in Go's pieces (the help, a
/// diagnostic). Only an fd 1 that was non-blocking at start waits on
/// EAGAIN; a later EAGAIN is the write's error, which the tsc writer
/// ignores as Go does. The trace output and the watch manager's lines
/// write here too. A write holds a lock for all of its partial writes, as
/// Go's `poll.FD.Write` holds the write lock of the file, so a line of
/// another thread never lands inside it.
///
/// PORT: when fd 1 is a regular file at start (`init`), the writes of a
/// thread inside a report (`keep_writes`) are kept. They go out together
/// when the report ends, at 64 KiB, before a write outside a report or to
/// `Stderr`, and in the panic hook and `throw`
/// (`flush_cli_stdout_at_exit`). Each other write goes out at once, in
/// Go's pieces, so a status line, a trace line or a `[watch]` line is in
/// the file when Go's is. A report only formats and writes, so it ends in
/// milliseconds. A process that dies inside one (SIGKILL, an OOM kill,
/// `core::go_fatal_newosproc`) loses the kept bytes, at most 64 KiB: Go's
/// file has the pieces written so far. A thrown signal and SIGHUP write
/// them first (bin/tsgo.rs `throw`, `die_from_signal`). The reports ignore
/// their write errors, as Go's do, so the error of a kept write changes
/// nothing.
pub struct CliStdout;

/// Go `os.Stderr`.
pub struct Stderr;

// Go: os/file.go:73 `Stdout = NewFile(...)`, at package init (go1.27.1)

/// fd 1 is a regular file: `CliStdout` keeps the writes of a report (see
/// `init`).
static COALESCE: AtomicBool = AtomicBool::new(false);

thread_local! {
    /// The number of `KeepWrites` of this thread that are live.
    static KEEPING: Cell<u32> = const { Cell::new(0) };
}

/// PORT: not in Go. One report: until the returned value drops, the writes
/// of `CliStdout` from this thread to a regular file are kept and go out
/// together (see `CliStdout`). The callers are reports that only format
/// and write: the diagnostics of an emit with its file list (also
/// `--explainFiles`) and error summary, the statistics table and the
/// `--showConfig` value. Do not call it around work that can wait.
#[must_use]
pub fn keep_writes() -> KeepWrites {
    KEEPING.with(|keeping| keeping.set(keeping.get() + 1));
    KeepWrites(PhantomData)
}

/// A live report (`keep_writes`). It stays on its thread.
pub struct KeepWrites(PhantomData<*const ()>);


/// `CliStdout` keeps the writes of this thread (see `keep_writes`).
fn keeping() -> bool {
    COALESCE.load(Ordering::Relaxed) && KEEPING.with(Cell::get) > 0
}

/// The write lock of `CliStdout` (Go `poll.FD.writeLock`), with the bytes
/// that it keeps (`keep_writes`).
static CLI_STDOUT: Mutex<Vec<u8>> = Mutex::new(Vec::new());

/// The kept bytes of `CliStdout` go out at this size.
const COALESCE_LIMIT: usize = 64 << 10;







// PORT: each write first writes what `CliStdout` keeps (`keep_writes`), so
// stdout and stderr keep their order in one file (`2>&1`).

#[cfg(unix)]
mod sys {
    use std::io;
    use std::sync::OnceLock;




    pub fn flush_stdout() -> io::Result<()> {
        Ok(())
    }

    // Go: internal/poll/fd_unix.go FD.Write




    // Go: internal/poll/fd_unix.go FD.Write and os/file.go File.Write
    // (`epipecheck`). The first write runs also for an empty `buf`. EAGAIN
    // waits only when fd 1 was non-blocking at start.

    pub fn flush_cli_stdout() -> io::Result<()> {
        Ok(())
    }



    // Go: runtime/signal_unix.go sigpipe and dieFromSignal. tsgo neither
    // ignores nor catches SIGPIPE through os/signal, so this always ends
    // the process. `emulate_default_handler` sets the default action,
    // unblocks the signal and raises it. When the raise returns (the pid 1
    // of a PID namespace: the kernel drops a signal with the default action
    // that the process sends itself), Go exits 128 + N. signal-hook aborts
    // there instead, which ends a pid 1 by SIGSEGV, so a pid 1 does not
    // raise the signal (as bin/tsgo.rs `end_by_signal`).
}

#[cfg(not(unix))]
mod sys {
    use std::io::{self, Read, Write};

    pub fn init() {}

    pub fn stdout_is_regular_file() -> bool {
        false
    }

    pub fn read_stdin(buf: &mut [u8]) -> io::Result<usize> {
        io::stdin().read(buf)
    }

    pub fn write_stdout(buf: &[u8]) -> io::Result<()> {
        io::stdout().write_all(buf)
    }

    pub fn flush_stdout() -> io::Result<()> {
        io::stdout().flush()
    }

    pub fn write_cli_stdout(buf: &[u8]) -> io::Result<()> {
        io::stdout().write_all(buf)
    }

    pub fn flush_cli_stdout() -> io::Result<()> {
        io::stdout().flush()
    }

    pub fn write_stderr(buf: &[u8]) -> io::Result<()> {
        io::stderr().write_all(buf)
    }
}
