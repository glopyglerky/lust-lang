//! Go: internal/fswatch/watcher.go (the `Watcher` API, the package
//! watchers, the shared backend base and the per-directory watch state).
//!
//! PORT: threads. Go runs one goroutine per backend event loop and one per
//! debouncer; callbacks run on the debouncer goroutine. The port uses
//! `std::thread` and shares state through `Arc`. Each Go `sync.Mutex` is a
//! `std::sync::Mutex` over the fields it guards (`*Locked` structs). A Go
//! `*dirWatch` map key is the `Arc` pointer (`Arc::as_ptr(..) as usize`).
//!
//! PORT: D-W1 (no `libc`, no `unsafe`). The Linux and kqueue backends call
//! the `fswatch::unix` shim of their targets (safe syscall crates). The
//! Windows backend uses the `notify` crate's ReadDirectoryChangesW watcher
//! (see windows.rs). The FSEvents backend is not ported: its package watcher
//! keeps a `None` factory, so `available()` is false, as on Linux in Go, and
//! `Default()` on macOS picks kqueue (Go's fallback when FSEvents is not
//! available).

use crate::fswatch::prelude::*;

use std::any::Any;
use std::panic::AssertUnwindSafe;
use std::sync::{Arc, Condvar, LazyLock, Mutex, OnceLock, Weak};
use std::time::Duration;

use crate::frontend::vfs::osvfs::{filepath_clean, };
use crate::fswatch::pathcompare::{ComparisonCache, ComparisonPath, PathComparer};
use crate::fswatch::syscall;
use crate::gostd::{errors, strconv};

// Go: watcher.go:17 errNilCallback

// Go: watcher.go:21 errRootPath

// Go: watcher.go:25 errNotAbsolute

// Go: watcher.go:31 ErrOverflow

// Go: watcher.go:37 ErrWatchTerminated

// Go: watcher.go:41 ErrUnavailable

// Go: watcher.go:50 ErrFilesystemUnsupported

// Go: watcher.go:58 Watcher

// Go: watcher.go:107 WatchOption

// Go: watcher.go:113 WatchDirectoryRequest

// Go: watcher.go:119 watchOptions

// Go: watcher.go:127 fileOption (ts#64210)
/// fileOption defers the file filter until the parent directory's comparer is
/// available, so WatchFile does not need a second filesystem query.
pub struct FileOption {
    pub path: String,
}


// Go: watcher.go:135 ignoreOption


// Go: watcher.go:147 WithIgnore

// Go: watcher.go:151 recursiveOption
pub struct RecursiveOption;


// Go: watcher.go:166 WithRecursive

// Go: watcher.go:172 Watch
/// Watch represents a live watch. Close stops watching
/// and releases resources. It is idempotent.
///
/// PORT: Go `io.Closer` users (watchmanager, lspwatcher) hold a
/// `Box<dyn Watch>`.
pub trait Watch: Send + Sync {
    fn close(&self) -> Result<(), GoError>;
    fn unexported(&self);
}

/// PORT: Go `error` in the callback. Plan decision D-CTX: one error type,
/// `GoError`; test sentinels with `errors::is(&err, &ERR_OVERFLOW)`.
pub type WatchError = GoError;

// Go: watcher.go:185 WatchCallback


// Go: watcher.go:188 package-level watcher instances
// Package-level watcher instances. Platform init() functions set the factory.
//
// PORT: Go package vars set up by the platform `init()` functions. The
// port builds each one on first use and calls the platform `init` there
// (inotify and fanotify on Linux, FSEvents on macOS, kqueue on darwin and
// the BSDs, windows on Windows).

// PORT: Go `&watcher{name: name}` composite literal, followed by the
// platform `init()` that sets the factory. The watcher also keeps a weak
// pointer to its own `Arc`, which Go gets from the `*watcher` receiver.

// Go: watcher.go:203 AllWatchers

// Go: watcher.go:214 Inotify

// Go: watcher.go:217 FSEvents

// Go: watcher.go:220 Kqueue

// Go: watcher.go:223 Windows

// Go: watcher.go:227 Fanotify

// Go: watcher.go:230 Default

// Go: watcher.go:255 fallbackWatcher


// Go: watcher.go:315 watcher


// Go: watcher.go:325 recursiveConsolidateThreshold
pub const RECURSIVE_CONSOLIDATE_THRESHOLD: usize = 10;



// Go: watcher.go:580 validateWatchDirectory

// Go: path/filepath/path_unix.go IsAbs
// PORT: Go standard library (unix). The crate's `filepath_clean` is the
// unix `filepath.Clean` too (the Windows one on Windows).
#[cfg(not(windows))]
fn filepath_is_abs(path: &str) -> bool {
    path.starts_with('/')
}

// Go: internal/filepathlite/path_windows.go IsAbs
// PORT: Go standard library (windows).
#[cfg(windows)]
fn filepath_is_abs(path: &str) -> bool {
    use crate::frontend::vfs::osvfs::{filepath_volume_name_len, win_is_path_separator};
    let b = path.as_bytes();
    let l = filepath_volume_name_len(b);
    if l == 0 {
        return false;
    }
    // If the volume name starts with a double slash, this is an absolute path.
    if win_is_path_separator(b[0]) && win_is_path_separator(b[1]) {
        return true;
    }
    let rest = &b[l..];
    !rest.is_empty() && win_is_path_separator(rest[0])
}

// Go: path/filepath/path.go Dir
// PORT: Go standard library (windows: `VolumeName` is the volume prefix
// with slashes made separators).
#[cfg(windows)]
fn filepath_dir(path: &str) -> String {
    use crate::frontend::vfs::osvfs::{filepath_volume_name_len, win_is_path_separator};
    let vol_len = filepath_volume_name_len(path.as_bytes());
    let vol = path[..vol_len].replace('/', "\\");
    let bytes = path.as_bytes();
    let mut i = bytes.len() as isize - 1;
    while i >= vol_len as isize && !win_is_path_separator(bytes[i as usize]) {
        i -= 1;
    }
    let dir = filepath_clean(&path[vol_len..(i + 1) as usize]);
    if dir == "." && vol.len() > 2 {
        // must be UNC
        return vol;
    }
    format!("{vol}{dir}")
}

// Go: path/filepath/path.go Dir
// PORT: Go standard library (unix: `VolumeName` is empty).
#[cfg(not(windows))]
fn filepath_dir(path: &str) -> String {
    let vol = "";
    let bytes = path.as_bytes();
    let mut i = bytes.len() as isize - 1;
    while i >= vol.len() as isize && bytes[i as usize] != b'/' {
        i -= 1;
    }
    let dir = filepath_clean(&path[vol.len()..(i + 1) as usize]);
    if dir == "." && vol.len() > 2 {
        // must be UNC
        return vol.to_string();
    }
    format!("{vol}{dir}")
}

// Go: watcher.go:611 watch


// Go: watcher.go:638 watcherImpl

/// PORT: Go `chan struct{}` that is only closed and received from (a
/// one-shot signal). Go replaces such a channel with `make` to reuse it;
/// the port makes a new `SignalChan`.
pub struct SignalChan {
    closed: Mutex<bool>,
    cond: Condvar,
}

impl Default for SignalChan {
    fn default() -> Self {
        SignalChan::new()
    }
}

impl SignalChan {
    /// Go `make(chan struct{})`.
    pub fn new() -> SignalChan {
        SignalChan {
            closed: Mutex::new(false),
            cond: Condvar::new(),
        }
    }

    /// Go `close(ch)`. Closing a closed channel panics, as in Go.
    pub fn close(&self) {
        let mut closed = self.closed.lock().unwrap();
        if *closed {
            panic!("close of closed channel");
        }
        *closed = true;
        self.cond.notify_all();
    }

    /// Go `select { case <-ch: ...; default: ... }`: true when closed.
    pub fn is_closed(&self) -> bool {
        *self.closed.lock().unwrap()
    }

    /// Go `<-ch`: blocks until the channel is closed.
    pub fn wait(&self) {
        let mut closed = self.closed.lock().unwrap();
        while !*closed {
            closed = self.cond.wait(closed).unwrap();
        }
    }

}

// Go: watcher.go:654 watcherBase



// ----- dirWatch: per-directory watch state -------------------------

// Go: watcher.go:782 callback

// Go: watcher.go:801 dirWatchError





// Go: watcher.go:811 dirWatch


// Go: watcher.go:834 newDirWatch
// PORT: `recursive` and `comparer` are parameters (see getOrCreateDirWatch):
// Go sets them on the new dirWatch before it is shared, the comparer with
// `setComparer`. The debounce key is the dirWatch pointer.

// Go: watcher.go:854 physicalDirFor

// Go: os.IsPathSeparator
// PORT: Go standard library. On unix only '/'; `MAIN_SEPARATOR` adds '\'
// on Windows, as Go does.
fn is_path_separator(c: u8) -> bool {
    c == b'/' || c == std::path::MAIN_SEPARATOR as u8
}

// Go: watcher.go:879 rebasePath
/// rebasePath replaces the from root in path with to, preserving any child
/// suffix. Prefix matches must end at a path separator so sibling paths like
/// "/foo2" are not rebased from "/foo".
pub fn rebase_path(path: &str, from: &str, to: &str) -> String {
    if from == to {
        return path.to_string();
    }
    if path == from {
        return to.to_string();
    }
    let Some(suffix) = path.strip_prefix(from) else {
        return path.to_string();
    };
    if !from.is_empty() && is_path_separator(from.as_bytes()[from.len() - 1]) {
        return join_path_suffix(to, suffix);
    }
    if suffix.is_empty() || !is_path_separator(suffix.as_bytes()[0]) {
        return path.to_string();
    }
    join_path_suffix(to, suffix)
}

// Go: watcher.go:899 joinPathSuffix
pub fn join_path_suffix(root: &str, suffix: &str) -> String {
    if suffix.is_empty() {
        return root.to_string();
    }
    let root_ends_with_separator =
        !root.is_empty() && is_path_separator(root.as_bytes()[root.len() - 1]);
    if is_path_separator(suffix.as_bytes()[0]) {
        if root_ends_with_separator {
            return format!("{root}{}", &suffix[1..]);
        }
        return format!("{root}{suffix}");
    }
    if root_ends_with_separator {
        return format!("{root}{suffix}");
    }
    format!("{root}{}{suffix}", std::path::MAIN_SEPARATOR)
}



// Go: watcher.go:1086 isInDirectoryOrSelf
pub fn is_in_directory_or_self(dir: &str, path: &str) -> bool {
    if dir.is_empty() {
        return false;
    }
    if path == dir {
        return true;
    }
    let Some(rest) = path.strip_prefix(dir) else {
        return false;
    };
    if rest.is_empty() {
        return false;
    }
    if is_path_separator(dir.as_bytes()[dir.len() - 1]) {
        return true;
    }
    is_path_separator(rest.as_bytes()[0])
}

// Go: watcher.go:1108 isDirectChild
/// isDirectChild reports whether path is an immediate child of dir.
/// Both paths must be absolute. Returns false for path == dir.
pub fn is_direct_child(dir: &str, path: &str) -> bool {
    if !path.starts_with(dir) {
        return false;
    }
    let rest = &path.as_bytes()[dir.len()..];
    if rest.is_empty() {
        return false;
    }
    let separator = std::path::MAIN_SEPARATOR as u8;
    if rest[0] != b'/' && rest[0] != separator {
        return false;
    }
    let rest = &rest[1..];
    !rest.is_empty() && !rest.contains(&b'/') && !rest.contains(&separator)
}

