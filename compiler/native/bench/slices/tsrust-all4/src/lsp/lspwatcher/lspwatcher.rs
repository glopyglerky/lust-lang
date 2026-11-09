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
pub trait WatcherBackend {
    fn watch_directory(
        &self,
        dir: &str,
        fn_: LocalWatchCallback,
        opts: &[Box<dyn fswatch::WatchOption>],
    ) -> Result<Box<dyn fswatch::Watch>, GoError>;
}

// Go: lsp/lspwatcher/lspwatcher.go:30 defaultWatcherBackend
// PORT: `bridge` is port-only. It moves fswatch callbacks to the dispatch
// thread (see `DeliveryBridge`).
pub struct DefaultWatcherBackend {
    pub watcher: Arc<dyn fswatch::Watcher>,
    pub bridge: Rc<DeliveryBridge>,
}


/// How often the dispatch thread takes the queued fswatch batches.
// PORT: no Go source. See `DeliveryBridge`.
const DELIVERY_POLL_INTERVAL: Duration = Duration::from_millis(10);

/// One fswatch callback call, queued for the dispatch thread.
struct Delivery {
    id: u64,
    events: Vec<fswatch::Event>,
    err: Option<GoError>,
}

/// Moves fswatch callbacks from the fswatch threads to the dispatch thread.
///
/// PORT: no Go source. Go calls the lspwatcher callbacks on the fswatch
/// goroutine. Here they touch dispatch-thread state (`Rc`), so the fswatch
/// callback queues each call in `inbox`, in arrival order. `gostd::local`
/// has no queue that another thread can post to, so a `local::after_func`
/// timer takes the queue every `DELIVERY_POLL_INTERVAL` while a
/// subscription is open, and calls each `LocalWatchCallback` in order. It
/// stops when all subscriptions are closed. A batch of a closed
/// subscription is dropped, as if Go's `Close` ran before the fswatch
/// callback.
#[derive(Default)]
pub struct DeliveryBridge {
    /// Filled by the fswatch threads.
    inbox: Arc<Mutex<VecDeque<Delivery>>>,
    /// Open subscriptions by id: the callback and the closed flag of its
    /// `BridgedWatch`.
    subscriptions: RefCell<FxHashMap<u64, (LocalWatchCallback, Arc<AtomicBool>)>>,
    next_id: Cell<u64>,
    poll_timer: RefCell<Option<local::LocalTimer>>,
}

impl DeliveryBridge {

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
pub struct Watcher {
    pub fs: Rc<dyn vfs::Fs>,
    pub backend: Box<dyn WatcherBackend>,
    pub on_changes: Box<dyn Fn(Vec<lsproto::FileEvent>)>,
    pub logger: Option<Rc<dyn logging::Logger>>,

    // watches holds the watches associated with each LSP WatcherID. A single id
    // may map to more than one watch because each FileSystemWatcher in the
    // registration becomes its own watch (different roots and kinds).
    pub watches: RefCell<IndexMap<String, Vec<Rc<Watch>>>>,
    pub closed: Cell<bool>,

    // Pending batch state, protected by mu.
    pub pending: RefCell<Option<IndexMap<String, lsproto::FileEvent>>>,
    pub flush_timer: RefCell<Option<local::LocalTimer>>,
}

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
pub struct Watch {
    pub watcher: Rc<Watcher>,
    pub requested_directory: String, // directory requested by the LSP layer (possibly a symlink)
    pub kind: lsproto::WatchKind,
    pub recursive: bool, // whether the target subscription should be recursive

    pub subscription: RefCell<Option<Box<dyn fswatch::Watch>>>, // current subscription (target or ancestor); nil if none
    pub watched_directory: RefCell<String>, // canonicalized directory 'subscription' is rooted at
    pub watching_target: Cell<bool>, // whether 'subscription' is rooted at the target directory
    pub closed: Cell<bool>,
}

// Go: lsp/lspwatcher/lspwatcher.go:95 New
// New constructs a Watcher backed by internal/fswatch's platform-default
// watcher implementation.

// Go: lsp/lspwatcher/lspwatcher.go:102 NewWithFSWatcher
// NewWithFSWatcher constructs a Watcher backed by the provided fswatch.Watcher.
// Use this to select a specific backend (e.g. fswatch.Kqueue()) instead of the
// platform default.

// Go: lsp/lspwatcher/lspwatcher.go:106 newWithBackend
pub fn new_with_backend(
    fs: Rc<dyn vfs::Fs>,
    backend: Box<dyn WatcherBackend>,
    on_changes: Box<dyn Fn(Vec<lsproto::FileEvent>)>,
    logger: Option<Rc<dyn logging::Logger>>,
) -> Rc<Watcher> {
    Rc::new(Watcher {
        fs,
        backend,
        on_changes,
        logger,
        watches: RefCell::new(IndexMap::new()),
        closed: Cell::new(false),
        pending: RefCell::new(None),
        flush_timer: RefCell::new(None),
    })
}

impl Watcher {
    // Go: lsp/lspwatcher/lspwatcher.go:125 Watcher.WatchFiles
    // WatchFiles subscribes to each FileSystemWatcher under the given id.
    //
    // A watcher whose directory does not exist yet is not an error: an ancestor
    // watch is installed on the nearest existing ancestor and the subscription is
    // reported as successful, so the session's notion of "this watcher is alive"
    // stays true for the subscription's whole lifetime. Only a genuine backend
    // failure (e.g. resource exhaustion while watching an existing directory)
    // causes WatchFiles to roll back the whole id and return an error, so the
    // session's pending/retry path re-registers it on the next reevaluation.

    // Go: lsp/lspwatcher/lspwatcher.go:179 Watcher.UnwatchFiles
    // UnwatchFiles tears down all subscriptions associated with id.

    // Go: lsp/lspwatcher/lspwatcher.go:195 Watcher.Close
    // Close removes every subscription. Safe to call multiple times.
    pub fn close(&self) {
        if self.closed.get() {
            return;
        }
        self.closed.set(true);
        let watches_by_id = std::mem::take(&mut *self.watches.borrow_mut());
        if let Some(flush_timer) = self.flush_timer.borrow_mut().take() {
            flush_timer.stop();
        }
        *self.pending.borrow_mut() = None;
        for watches in watches_by_id.values() {
            for watch in watches {
                watch.close();
            }
        }
    }

    // Go: lsp/lspwatcher/lspwatcher.go:396 Watcher.forwardEvents
    // forwardEvents translates fswatch events into LSP file events and enqueues
    // them for the next debounced flush.

    // Go: lsp/lspwatcher/lspwatcher.go:442 Watcher.emitSyntheticCreates
    // emitSyntheticCreates enqueues synthetic create events after a target watch is
    // (re)installed following a missing→present transition, so the session
    // re-resolves files that appeared before the real watch existed. The target
    // directory itself is always included; for a non-recursive watch its immediate
    // children are added, and for a recursive watch its whole subtree is walked.
    // Nothing is emitted if the watch doesn't request create notifications.

    // Go: lsp/lspwatcher/lspwatcher.go:469 Watcher.enqueueSyntheticCreates
    // enqueueSyntheticCreates adds synthetic create events for paths, without
    // clobbering a more specific event already pending for the same path (e.g. a
    // real delete).

    // Go: lsp/lspwatcher/lspwatcher.go:494 Watcher.scheduleFlushLocked
    // scheduleFlushLocked arms the debounce flush timer if it isn't already armed.
    // Callers must hold w.mu.

    // Go: lsp/lspwatcher/lspwatcher.go:500 Watcher.flush
}

impl Watch {
    // Go: lsp/lspwatcher/lspwatcher.go:219 watch.close
    // close tears down the watch's current subscription and prevents any in-flight
    // reconcile from reinstalling one.
    pub fn close(&self) {
        self.closed.set(true);
        let subscription = self.subscription.borrow_mut().take();
        *self.watched_directory.borrow_mut() = String::new();
        if let Some(subscription) = subscription {
            let _ = subscription.close();
        }
    }

    // Go: lsp/lspwatcher/lspwatcher.go:244 watch.reconcile
    // reconcile installs or advances this watch toward the target directory based
    // on the current filesystem state. It is called at registration, whenever a
    // ancestor watch observes activity, and after a target watch is terminated by
    // deletion.
    //
    // emitSynthetic controls whether promoting to the target emits synthetic
    // create events: false for the initial install when the target already exists
    // (the session already knows about those files), true for any missing→present
    // recovery.
    //
    // It returns a non-nil error only on a genuine backend failure to install a
    // watch; a missing target directory is handled by installing an ancestor watch
    // and returns nil.

    // Go: lsp/lspwatcher/lspwatcher.go:318 watch.targetCallback
    // targetCallback returns the fswatch callback for a target watch rooted at
    // watchedReal. It forwards events to the session and, on ErrWatchTerminated
    // (the watched directory was deleted), falls back to watching the nearest
    // existing ancestor so the watch re-attaches when the directory is recreated.

    // Go: lsp/lspwatcher/lspwatcher.go:351 watch.handleTerminated
    // handleTerminated clears the dead target watch (the backend has already
    // removed it) and re-evaluates, falling back to an ancestor watch on the nearest
    // existing ancestor so the watch re-attaches when the directory reappears.
    // Clearing the state first is essential: reconcile would otherwise see
    // watchingTarget && subscription != nil and conclude the target is already
    // watched, even though the subscription is dead — losing recovery if the
    // directory is recreated before reconcile runs.

    // Go: lsp/lspwatcher/lspwatcher.go:372 watch.ancestorCallback
    // ancestorCallback returns the fswatch callback for an ancestor watch. Ancestor
    // watches exist only to detect the target — or an intermediate path component —
    // being created; their events are about ancestor directories the session
    // doesn't track, so they are ignored and the watch is simply re-evaluated.
}

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
