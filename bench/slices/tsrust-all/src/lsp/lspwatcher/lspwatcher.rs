//! Go `internal/lsp/lspwatcher/lspwatcher.go`.
//!
//! Package lspwatcher implements an in-process file watcher used as a
//! drop-in replacement for LSP-based file watching when the client does not
//! support dynamic registration of file watchers.
//!
//! PORT: the watcher holds the session file system (`Rc<dyn vfs::Fs>`), so
//! it lives on the LSP dispatch thread (PORTING "Threads"). Go mutexes are
//! dropped; state is in `Cell` / `RefCell`. Go `*Watcher` and `*watch` are
//! `Rc`. `time.AfterFunc` is `gostd::local::after_func`, so `flush` runs on
//! the dispatch thread. fswatch callbacks reach the dispatch thread through
//! `DeliveryBridge`. The server only makes this watcher when
//! `fswatch::default().has_fast_recursive_backend()` is true: on macOS
//! (FSEvents) and Windows, not on Linux.

use crate::lsp::lspwatcher::prelude::*;

use crate::frontend::{tspath, vfs};
use crate::fswatch;
use crate::gostd::{errors, local, strconv};
use crate::ls::lsconv;
use crate::lsp::lsproto;
use crate::project::logging;
use crate::project::logging::Logger as _;
use std::cell::Cell;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

// Go: lsp/lspwatcher/lspwatcher.go:24 throttleWindow
// throttleWindow mirrors VS Code's parcel watcher integration: give the
// first batch a short grace window so adjacent filesystem bursts coalesce.
pub const THROTTLE_WINDOW: Duration = Duration::from_millis(75);

/// The callback a `WatcherBackend` subscription calls.
// PORT: Go passes an `fswatch.WatchCallback`. The lspwatcher callbacks touch
// the file system, logger and subscriptions of the watcher, which are
// dispatch-thread state (`Rc`), so they are not `Send`.
pub type LocalWatchCallback = Rc<dyn Fn(Vec<fswatch::Event>, Option<GoError>)>;

// Go: lsp/lspwatcher/lspwatcher.go:26 watcherBackend
// PORT: Go `io.Closer` is `Box<dyn fswatch::Watch>`; the variadic options
// are a slice. The callback is a `LocalWatchCallback` (see above).

// Go: lsp/lspwatcher/lspwatcher.go:30 defaultWatcherBackend
// PORT: `bridge` is port-only. It moves fswatch callbacks to the dispatch
// thread (see `DeliveryBridge`).


/// How often the dispatch thread takes the queued fswatch batches.
// PORT: no Go source. See `DeliveryBridge`.
const DELIVERY_POLL_INTERVAL: Duration = Duration::from_millis(10);

/// One fswatch callback call, queued for the dispatch thread.
struct Delivery {
    id: u64,
    events: Vec<fswatch::Event>,
    err: Option<GoError>,
}



/// The `fswatch::Watch` that `DefaultWatcherBackend` returns: closing it
/// also stops the delivery of its queued batches.
struct BridgedWatch {
    inner: Box<dyn fswatch::Watch>,
    closed: Arc<AtomicBool>,
}

impl fswatch::Watch for BridgedWatch {
    fn close(&self) -> Result<(), GoError> {
        self.closed.store(true, Ordering::SeqCst);
        self.inner.close()
    }

    fn unexported(&self) {}
}

// Go: lsp/lspwatcher/lspwatcher.go:42 Watcher
// Watcher manages a set of file system subscriptions identified by
// WatcherID strings (matching the LSP server's project.WatcherID type).
// Events are delivered to onChanges in batches as `*lsproto.FileEvent`,
// shaped exactly like a `workspace/didChangeWatchedFiles` notification.
// PORT: Go map fields are `IndexMap`s (insertion order; Go map order is
// random). A nil map after `Close` is an empty one.

// Go: lsp/lspwatcher/lspwatcher.go:80 watch
// watch represents one FileSystemWatcher from the LSP registration.
//
// The directory the session asks to watch may not exist yet (common in
// granular mode, where each probed-but-missing package directory becomes a
// watch) or may be deleted while watched. To honor the watch across those
// transitions, a watch maintains either:
//
//   - a "target" subscription rooted directly at the requested directory, once
//     it exists, or
//   - an "ancestor" subscription on the nearest existing ancestor
//     (non-recursive), used to detect the requested directory — or an
//     intermediate path component — being created, after which the watch
//     descends toward and eventually promotes to the target.
//
// When the target materializes, synthetic create events are emitted for it
// (and, depending on whether the watch is recursive, its immediate children or
// its whole subtree) so the session re-resolves files that appeared in the gap
// before the real subscription was installed.
//
// All path fields are tspath-style (forward-slash) absolute paths.

// Go: lsp/lspwatcher/lspwatcher.go:95 New
// New constructs a Watcher backed by internal/fswatch's platform-default
// watcher implementation.

// Go: lsp/lspwatcher/lspwatcher.go:102 NewWithFSWatcher
// NewWithFSWatcher constructs a Watcher backed by the provided fswatch.Watcher.
// Use this to select a specific backend (e.g. fswatch.Kqueue()) instead of the
// platform default.

// Go: lsp/lspwatcher/lspwatcher.go:106 newWithBackend



// Go: lsp/lspwatcher/lspwatcher.go:381 nearestExistingAncestor
// nearestExistingAncestor returns the deepest existing directory that is dir or
// an ancestor of dir, walking upward. ok is false only if nothing in the chain
// (including the root) exists.

// Go: lsp/lspwatcher/lspwatcher.go:531 watchRoot
// watchRoot extracts the directory the fswatch subscription should be
// rooted at from a FileSystemWatcher. The patterns the project layer
// produces are of the form `<dir>/**/*` (recursive) or `<dir>/*`
// (non-recursive, used by granular watch mode), either as a Pattern
// with a fully-qualified directory or as a RelativePattern with a
// file:// BaseUri, so the heuristic of "everything before the first
// glob meta character" is reliable. Use [isRecursiveGlob] to determine
// whether the subscription should be recursive.
//
// Returned roots are tspath-normalized (forward-slash) absolute paths.

// Go: lsp/lspwatcher/lspwatcher.go:548 rootFromGlob

// Go: lsp/lspwatcher/lspwatcher.go:570 watchPatternString
pub fn watch_pattern_string(file_system_watcher: &lsproto::FileSystemWatcher) -> String {
    if let Some(pattern) = &file_system_watcher.glob_pattern.pattern {
        return pattern.clone();
    }
    if let Some(relative_pattern) = &file_system_watcher.glob_pattern.relative_pattern {
        let mut base = String::new();
        if let Some(uri) = &relative_pattern.base_uri.uri {
            base = uri.0.clone();
        }
        return base + "/" + &relative_pattern.pattern;
    }
    String::new()
}

// Go: lsp/lspwatcher/lspwatcher.go:587 isRecursiveGlob
// isRecursiveGlob reports whether a FileSystemWatcher's pattern requests
// recursive watching (contains a `**` segment). Granular watch mode emits
// non-recursive `<dir>/*` patterns, which watch only the immediate directory.
pub fn is_recursive_glob(file_system_watcher: &lsproto::FileSystemWatcher) -> bool {
    watch_pattern_string(file_system_watcher).contains("**")
}

// Go: lsp/lspwatcher/lspwatcher.go:591 effectiveKind
pub fn effective_kind(file_system_watcher: &lsproto::FileSystemWatcher) -> lsproto::WatchKind {
    if let Some(kind) = file_system_watcher.kind {
        return kind;
    }
    lsproto::WatchKind(
        lsproto::WatchKind::CREATE.0 | lsproto::WatchKind::CHANGE.0 | lsproto::WatchKind::DELETE.0,
    )
}
