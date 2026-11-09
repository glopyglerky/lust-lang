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
/// Watcher represents a filesystem watching implementation.
/// Use one of the constructor functions ([Inotify], [FSEvents], [Kqueue],
/// [Windows]) to obtain a value, or [Default] for the platform default.
///
/// All watchers exist on every platform. Subscribing with a watcher that
/// is not supported on the current OS returns [ErrUnavailable].
///
/// PORT: Go `WatchDirectory` checks `fn == nil`; a Rust `WatchCallback` is
/// never nil, so `ERR_NIL_CALLBACK` is not returned. Go returns the
/// `Watch` interface; the port returns `Box<dyn Watch>`.
pub trait Watcher: Send + Sync {
    /// Name returns a stable identifier ("inotify", "fsevents", "kqueue",
    /// "windows").
    fn name(&self) -> String;
    /// Available reports whether this watcher works on the current OS.
    fn available(&self) -> bool;
    /// HasFastRecursiveBackend reports whether this watcher supports efficient
    /// recursive watching without requiring a full userspace tree walk. This is
    /// true for Windows (ReadDirectoryChangesW subtree mode) and macOS FSEvents
    /// (inherently recursive), and false for all other backends.
    fn has_fast_recursive_backend(&self) -> bool;
    /// WatchDirectory watches dir for changes, calling fn with batched
    /// events. By default, only direct children are watched. Use
    /// [WithRecursive] to watch the entire directory tree.
    /// dir must be an absolute path to an existing directory. If dir is a
    /// symlink or reparse point to a directory, the OS subscription follows
    /// the target directory but delivered event paths remain rooted at dir.
    /// Userspace recursive traversal does not follow symlinked descendant
    /// directories.
    /// Returns [ErrUnavailable] if the watcher is not supported on
    /// the current platform.
    fn watch_directory(
        &self,
        dir: &str,
        fn_: WatchCallback,
        opts: &[Box<dyn WatchOption>],
    ) -> Result<Box<dyn Watch>, GoError>;
    /// WatchDirectories watches multiple directories as a batch. It has the
    /// same semantics as calling [Watcher.WatchDirectory] for each request, but
    /// lets backends arm the underlying OS watches once for the whole batch.
    /// Returned watches are in the same order as requests.
    fn watch_directories(
        &self,
        requests: &[WatchDirectoryRequest<'_>],
    ) -> Result<Vec<Box<dyn Watch>>, GoError>;
    /// WatchFile watches a single file for changes, calling fn with
    /// batched events. path must be an absolute path. The file does not
    /// need to exist at subscribe time; its creation will be reported.
    /// The parent directory must exist.
    ///
    /// Multiple WatchFile calls for files in the same directory
    /// share a single OS watch on the parent directory.
    ///
    /// If the parent directory is deleted, [ErrWatchTerminated] is
    /// delivered and the watch is dead. Unlike TypeScript's
    /// watchFile (which falls back to polling for missing entries),
    /// there is no automatic recovery. Callers that need to survive
    /// parent directory deletion should handle [ErrWatchTerminated]
    /// and re-subscribe when the directory is recreated.
    ///
    /// Returns [ErrUnavailable] if the watcher is not supported on
    /// the current platform.
    fn watch_file(&self, path: &str, fn_: WatchCallback) -> Result<Box<dyn Watch>, GoError>;
    fn unexported(&self);
}

// Go: watcher.go:107 WatchOption
/// WatchOption configures a watch.
pub trait WatchOption: Send + Sync {
    fn apply_watch_option(&self, opts: &mut WatchOptions);
}

// Go: watcher.go:113 WatchDirectoryRequest
/// WatchDirectoryRequest describes one directory subscription in a
/// [Watcher.WatchDirectories] batch.
///
/// PORT: the Go `[]WatchOption` slice is a borrowed slice.
pub struct WatchDirectoryRequest<'a> {
    pub dir: String,
    pub callback: WatchCallback,
    pub options: &'a [Box<dyn WatchOption>],
}

// Go: watcher.go:119 watchOptions
#[derive(Clone, Default)]
pub struct WatchOptions {
    pub ignore: Option<Arc<dyn Fn(&str) -> bool + Send + Sync>>,
    pub recursive: bool,
    // ts#64210
    pub file: String,
}

// Go: watcher.go:127 fileOption (ts#64210)
/// fileOption defers the file filter until the parent directory's comparer is
/// available, so WatchFile does not need a second filesystem query.
pub struct FileOption {
    pub path: String,
}

impl WatchOption for FileOption {
    // Go: watcher.go:131 fileOption.applyWatchOption
    fn apply_watch_option(&self, opts: &mut WatchOptions) {
        opts.file = self.path.clone();
    }
}

// Go: watcher.go:135 ignoreOption
pub struct IgnoreOption {
    pub fn_: Arc<dyn Fn(&str) -> bool + Send + Sync>,
}

impl WatchOption for IgnoreOption {
    // Go: watcher.go:139 ignoreOption.applyWatchOption
    fn apply_watch_option(&self, opts: &mut WatchOptions) {
        opts.ignore = Some(self.fn_.clone());
    }
}

// Go: watcher.go:147 WithIgnore
/// WithIgnore returns a [WatchOption] that filters events before delivery.
/// If the function returns true for a path, events for that path are
/// silently dropped. The filtering is per-subscriber; multiple watches
/// on the same directory may have different ignore functions.
pub fn with_ignore(fn_: Arc<dyn Fn(&str) -> bool + Send + Sync>) -> Box<dyn WatchOption> {
    Box::new(IgnoreOption { fn_ })
}

// Go: watcher.go:151 recursiveOption
pub struct RecursiveOption;

impl WatchOption for RecursiveOption {
    // Go: watcher.go:153 recursiveOption.applyWatchOption
    fn apply_watch_option(&self, opts: &mut WatchOptions) {
        opts.recursive = true;
    }
}

// Go: watcher.go:166 WithRecursive
/// WithRecursive returns a [WatchOption] that enables recursive watching
/// of the entire directory tree. Without this option,
/// [Watcher.WatchDirectory] watches only direct children of dir.
///
/// In recursive mode, events for all descendants at any depth are
/// delivered. On inotify/fanotify, a watch descriptor is added for
/// every subdirectory. On kqueue, an fd is opened for every entry.
/// On Windows, bWatchSubtree=TRUE is passed to ReadDirectoryChangesW.
/// On FSEvents, the kernel is inherently recursive.
pub fn with_recursive() -> Box<dyn WatchOption> {
    Box::new(RecursiveOption)
}

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
/// WatchCallback receives batched filesystem events. Rapid changes
/// are coalesced before delivery.
///
/// For a given Watch, the callback is never invoked concurrently
/// with itself. It runs on a library goroutine, not the caller's.
///
/// When err is non-nil, use [errors.Is] to check for [ErrOverflow]
/// (recoverable) or [ErrWatchTerminated] (terminal).
///
/// PORT: Go `nil` events are an empty `Vec`.
pub type WatchCallback = Arc<dyn Fn(Vec<Event>, Option<WatchError>) + Send + Sync>;


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
/// fallbackWatcher keeps the primary backend for supported filesystems while
/// routing individual unsupported watches to the secondary backend.
pub struct FallbackWatcher {
    pub primary: Arc<dyn Watcher>,
    pub secondary: Arc<dyn Watcher>,
}


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

/// PORT: the `watcherBase` fields that Go `b.mu` guards.
#[derive(Default)]
pub struct WatcherBaseLocked {
    /// Go `map[*dirWatch]struct{}`, keyed by the `Arc` pointer.
    pub subscriptions: FxHashMap<usize, Arc<DirWatch>>,
    pub start_err: Option<GoError>,
}


// ----- dirWatch: per-directory watch state -------------------------

// Go: watcher.go:782 callback
#[derive(Clone)]
pub struct Callback {
    pub id: u64,
    pub dir: String,
    pub physical_dir: String,
    pub watch_dir: String,
    pub watch_physical_dir: String,
    pub recursive: bool,
    pub fn_: WatchCallback,
    pub ignore: Option<Arc<dyn Fn(&str) -> bool + Send + Sync>>,
    pub since_seq: u64,
    pub terminal: Option<GoError>,
    pub delivered: bool,
    // ts#64210
    pub comparer: PathComparer,
    pub dir_comparison: ComparisonPath<'static>,
    pub physical_comparison: ComparisonPath<'static>,
    pub file_comparison: ComparisonPath<'static>,
}

// Go: watcher.go:801 dirWatchError
/// dirWatchError associates an error with a specific directory watch.
#[derive(Clone)]
pub struct DirWatchError {
    pub err: GoError,
    pub dir_watch: Arc<DirWatch>,
}

impl DirWatchError {
    // Go: watcher.go:806 dirWatchError.Error
    pub fn error(&self) -> String {
        self.err.error()
    }

    // Go: watcher.go:807 dirWatchError.Unwrap
    pub fn unwrap(&self) -> GoError {
        self.err.clone()
    }

}

impl std::fmt::Display for DirWatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.error())
    }
}

impl std::fmt::Debug for DirWatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "dirWatchError({:?}, {:?})",
            self.dir_watch.dir,
            self.err.error()
        )
    }
}

/// PORT: Go compares `*dirWatchError` pointers. The port compares the
/// wrapped error value and the dirWatch pointer.
impl PartialEq for DirWatchError {
    fn eq(&self, other: &DirWatchError) -> bool {
        self.err == other.err && Arc::ptr_eq(&self.dir_watch, &other.dir_watch)
    }
}

// Go: watcher.go:811 dirWatch
/// dirWatch holds per-directory state: pending events, registered callbacks,
/// and a reference to the shared debouncer. Each watched directory has one.
///
/// PORT: Go `mu` guards the fields in `DirWatchLocked`. `state` is only
/// used by the fsevents and Windows backends. Go
/// `sequence` is always nil here (see `WatcherStruct`), so it is not a field.
/// Go sets `comparer`, `dirFold` and `physicalDirFold` with `setComparer`
/// before the dirWatch is shared; the port sets them in `new_dir_watch`.
pub struct DirWatch {
    /// dir is the caller-visible watch root used in delivered event paths.
    pub dir: String,
    /// physicalDir is the path passed to OS watcher APIs. It differs from dir
    /// when dir or an ancestor is a symlink or reparse point to a directory.
    pub physical_dir: String,
    pub recursive: bool,
    pub events: EventList,
    // ts#64210
    pub comparer: PathComparer,
    pub dir_fold: String,
    pub physical_dir_fold: String,

    /// state stores per-directory platform-specific bookkeeping (fsevents, windows).
    pub state: Mutex<Option<Box<dyn Any + Send>>>,

    pub mu: Mutex<DirWatchLocked>,
}

/// PORT: the `dirWatch` fields that Go `dw.mu` guards.
#[derive(Default)]
pub struct DirWatchLocked {
    pub callbacks: Vec<Callback>,
    pub debounce: Option<Arc<Debounce>>,
    pub next_cbid: u64,
}

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

impl DirWatch {
    // Go: watcher.go:867 dirWatch.displayPath
    /// displayPath maps a physical event path back under the caller-visible
    /// watch root.
    pub fn display_path(&self, watch_path: &str) -> String {
        rebase_path(watch_path, &self.physical_dir, &self.dir)
    }

    // Go: watcher.go:872 dirWatch.physicalPath
    /// physicalPath maps a caller-visible path to the physical watched root.
    pub fn physical_path(&self, display_path: &str) -> String {
        rebase_path(display_path, &self.dir, &self.physical_dir)
    }

    // Go: watcher.go:915 dirWatch.destroyDebounce

    // Go: watcher.go:925 dirWatch.notify

    // Go: watcher.go:943 dirWatch.notifyError
    pub fn notify_error(&self, err: GoError) {
        let cbs = {
            let mut dw = self.mu.lock().unwrap();
            let cbs = dw.callbacks.clone();
            dw.callbacks = Vec::new();
            cbs
        };
        for cb in &cbs {
            (cb.fn_)(Vec::new(), Some(err.clone()));
        }
    }

    // Go: watcher.go:953 dirWatch.triggerCallbacks

    // Go: watcher.go:1063 dirWatch.terminateCallbacksForDeletedRoot
}

impl Callback {
    // Go: watcher.go:1038 callback.mapEvent

    // Go: watcher.go:1042 callback.mapEventCached (ts#64210)

    // Go: watcher.go:1056 callback.eventPhysicalPath
    pub fn event_physical_path(&self, path: &str) -> String {
        if !self.watch_physical_dir.is_empty()
            && !self.watch_dir.is_empty()
            && self.watch_physical_dir != self.watch_dir
            && is_in_directory_or_self(&self.watch_dir, path)
        {
            return rebase_path(path, &self.watch_dir, &self.watch_physical_dir);
        }
        path.to_string()
    }
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

impl DirWatch {
    // Go: watcher.go:1123 dirWatch.watch
    pub fn watch(
        &self,
        dir: &str,
        physical_dir: &str,
        recursive: bool,
        fn_: WatchCallback,
        ignore: Option<Arc<dyn Fn(&str) -> bool + Send + Sync>>,
    ) -> (u64, bool) {
        self.add_callback(dir, physical_dir, recursive, fn_, ignore, "")
    }

    // Go: watcher.go:1127 dirWatch.addCallback (ts#64210)
    // PORT: Go reads `dw.sequence()` when it is set; it is always nil here
    // (see `WatcherStruct`).
    pub fn add_callback(
        &self,
        dir: &str,
        physical_dir: &str,
        recursive: bool,
        fn_: WatchCallback,
        ignore: Option<Arc<dyn Fn(&str) -> bool + Send + Sync>>,
        file: &str,
    ) -> (u64, bool) {
        let mut dw = self.mu.lock().unwrap();
        dw.next_cbid += 1;
        let id = dw.next_cbid;
        let since_seq = self.events.sequence();
        dw.callbacks.push(Callback {
            id,
            dir: dir.to_string(),
            physical_dir: physical_dir.to_string(),
            watch_dir: self.dir.clone(),
            watch_physical_dir: self.physical_dir.clone(),
            recursive,
            fn_,
            ignore,
            since_seq,
            terminal: None,
            delivered: false,
            comparer: self.comparer,
            dir_comparison: self.comparer.prepare(dir),
            physical_comparison: self.comparer.prepare(physical_dir),
            file_comparison: self.comparer.prepare(file),
        });
        (id, true)
    }

    // Go: watcher.go:1145 dirWatch.unwatch
    pub fn unwatch(&self, id: u64) -> bool {
        let mut dw = self.mu.lock().unwrap();
        for i in 0..dw.callbacks.len() {
            if dw.callbacks[i].id == id {
                dw.callbacks.remove(i);
                return dw.callbacks.is_empty();
            }
        }
        false
    }

    // Go: watcher.go:1157 dirWatch.unref
}
