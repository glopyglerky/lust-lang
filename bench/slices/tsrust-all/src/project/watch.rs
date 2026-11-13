//! Go `internal/project/watch.go`.
//!
//! PORT: one thread (project/dirty/interfaces.rs). Go mutexes are dropped;
//! state that Go writes after sharing is in `Cell` / `RefCell`. Go
//! `map[string]struct{}` is `FxHashSet<String>` (a nil map is empty). Go
//! `[]*lsproto.FileSystemWatcher` is `Vec<lsproto::FileSystemWatcher>`.

use crate::project::prelude::*;

use crate::frontend::stringutil_ls;
use std::cell::Cell;
use std::sync::atomic::{AtomicU64, Ordering};

// Go: project/watch.go:19 minWatchLocationDepth
pub const MIN_WATCH_LOCATION_DEPTH: i32 = 2;

// Go: project/watch.go:22 fileSystemWatcherKey
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct FileSystemWatcherKey {
    pub pattern: String,
    pub kind: lsproto::WatchKind,
}

// Go: project/watch.go:27 fileSystemWatcherValue
#[derive(Clone, Debug, Default)]
pub struct FileSystemWatcherValue {
    pub count: i32,
    pub id: WatcherID,
}

// Go: project/watch.go:37 watchRegistry
// watchRegistry tracks the current watch globs and how many individual
// WatchedFiles reference each glob. It provides ref-count helpers so callers
// don't manipulate the map directly.
//
// All methods are safe for concurrent use; locking is handled internally.
// PORT: `mu` is dropped (one thread); the maps are `RefCell`s. Go
// `*fileSystemWatcherValue` map values are plain values (never shared).
#[derive(Debug, Default)]
pub struct WatchRegistry {
    pub entries: RefCell<FxHashMap<FileSystemWatcherKey, FileSystemWatcherValue>>,
    pub pending: RefCell<FxHashSet<WatcherID>>,
}

// Go: project/watch.go:43 newWatchRegistry
pub fn new_watch_registry() -> Rc<WatchRegistry> {
    Rc::new(WatchRegistry {
        entries: RefCell::new(FxHashMap::default()),
        pending: RefCell::new(FxHashSet::default()),
    })
}

impl WatchRegistry {
    // Go: project/watch.go:53 watchRegistry.Acquire
    // Acquire increments the ref count for a watcher. If this is the first
    // reference (count goes from 0 to 1), it returns true so the caller knows
    // to register the watcher with the client.

    // Go: project/watch.go:69 watchRegistry.Release
    // Release decrements the ref count for a watcher. If no references remain,
    // the entry is removed and the function returns the WatcherID and true so
    // the caller knows to unregister the watcher from the client.
    pub fn release(&self, watcher: &lsproto::FileSystemWatcher) -> (WatcherID, bool) {
        let key = to_file_system_watcher_key(watcher);
        let mut entries = self.entries.borrow_mut();
        let Some(value) = entries.get_mut(&key) else {
            return (WatcherID::default(), false);
        };
        if value.count <= 1 {
            let id = value.id.clone();
            entries.remove(&key);
            return (id, true);
        }
        value.count -= 1;
        (WatcherID::default(), false)
    }

    // Go: project/watch.go:86 watchRegistry.MarkPending
    // MarkPending records that a watcher's registration failed and needs retry.
    pub fn mark_pending(&self, id: WatcherID) {
        self.pending.borrow_mut().insert(id);
    }

    // Go: project/watch.go:93 watchRegistry.ClearPending
    // ClearPending removes a watcher from the pending set after successful registration.
    pub fn clear_pending(&self, id: &WatcherID) {
        self.pending.borrow_mut().remove(id);
    }

    // Go: project/watch.go:100 watchRegistry.IsPending
    // IsPending returns true if the watcher needs retry due to a previous failure.
    pub fn is_pending(&self, id: &WatcherID) -> bool {
        self.pending.borrow().contains(id)
    }
}

// Go: project/watch.go:107 PatternsAndIgnored
#[derive(Clone, Debug, Default)]
pub struct PatternsAndIgnored {
    pub directories_outside_workspace: Vec<String>,
    pub patterns_inside_workspace: Vec<String>,
    pub ignored: FxHashSet<String>,
}

// Go: project/watch.go:119 toFileSystemWatcherKey
// toFileSystemWatcherKey produces a deduplication key for a file system watcher.
// Note: this key is a simple string concatenation of the base and pattern, so
// structurally different watchers (Pattern vs RelativePattern, URI vs WorkspaceFolder)
// could theoretically collide. In practice, workspace watchers use plain Pattern
// with filesystem paths while outside-workspace watchers use RelativePattern with
// file:// URIs, so collisions don't occur.
pub fn to_file_system_watcher_key(w: &lsproto::FileSystemWatcher) -> FileSystemWatcherKey {
    let kind = match w.kind {
        Some(kind) => kind,
        None => lsproto::WatchKind(
            lsproto::WatchKind::CREATE.0
                | lsproto::WatchKind::CHANGE.0
                | lsproto::WatchKind::DELETE.0,
        ),
    };
    let mut pattern = String::new();
    if let Some(p) = &w.glob_pattern.pattern {
        pattern = p.clone();
    } else if let Some(relative_pattern) = &w.glob_pattern.relative_pattern {
        let mut base = String::new();
        if let Some(uri) = &relative_pattern.base_uri.uri {
            base = uri.0.clone();
        } else if relative_pattern.base_uri.workspace_folder.is_some() {
            crate::core::go_panic(
                "workspace folder-based relative patterns not implemented".to_string(),
            );
        }
        pattern = base + "/" + &relative_pattern.pattern;
    }
    FileSystemWatcherKey { pattern, kind }
}

// Go: project/watch.go:139 fileSystemWatcherGlobString
pub fn file_system_watcher_glob_string(w: &lsproto::FileSystemWatcher) -> String {
    if let Some(pattern) = &w.glob_pattern.pattern {
        return pattern.clone();
    }
    if let Some(relative_pattern) = &w.glob_pattern.relative_pattern {
        let mut base = String::new();
        if let Some(uri) = &relative_pattern.base_uri.uri {
            base = uri.0.clone();
        } else if relative_pattern.base_uri.workspace_folder.is_some() {
            crate::core::go_panic(
                "workspace folder-based relative patterns not implemented".to_string(),
            );
        }
        return base + "/" + &relative_pattern.pattern;
    }
    String::new()
}

// Go: project/watch.go:155 WatcherID
// PORT: Go `type WatcherID string`; the empty id is Go's "".
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WatcherID(pub String);

// Go `%s` / `%v` of a string type prints the string.
impl std::fmt::Display for WatcherID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

// Go: project/watch.go:157 watcherID
// PORT: Go `atomic.Uint64`: one process-wide counter that starts at 0. Ids
// reach the client (registration ids), so every thread must share it.
pub static WATCHER_ID: AtomicU64 = AtomicU64::new(0);

/// Go `watcherID.Add(delta)`: adds `delta` and returns the new value. Go
/// atomics are sequentially consistent and wrap on overflow.
fn watcher_id_add(delta: u64) -> u64 {
    WATCHER_ID
        .fetch_add(delta, Ordering::SeqCst)
        .wrapping_add(delta)
}

// Go: project/watch.go:159 WatchedFiles
// PORT: Go `*WatchedFiles[T]` is `Rc<WatchedFiles<T>>` (nil is `None`).
// `mu` is dropped; `computeWatchersOnce` is a `Cell<bool>` guard, and the
// fields that `Watchers` writes are `Cell` / `RefCell`.
pub struct WatchedFiles<T> {
    pub name: String,
    pub watch_kind: lsproto::WatchKind,
    pub has_relative_pattern_capability: bool,
    pub compute_glob_patterns: Rc<dyn Fn(&T) -> PatternsAndIgnored>,

    pub input: T,
    pub compute_watchers_once: Cell<bool>,
    pub workspace_watchers: RefCell<Vec<lsproto::FileSystemWatcher>>,
    pub outside_workspace_watchers: RefCell<Vec<lsproto::FileSystemWatcher>>,
    pub ignored: RefCell<FxHashSet<String>>,
    pub id: Cell<u64>,
}

// Go: project/watch.go:174 NewWatchedFiles
// PORT: Go leaves `input` at the zero value of T (`T::default()`).
pub fn new_watched_files<T: Default>(
    name: &str,
    watch_kind: lsproto::WatchKind,
    has_relative_pattern_capability: bool,
    compute_glob_patterns: Rc<dyn Fn(&T) -> PatternsAndIgnored>,
) -> Rc<WatchedFiles<T>> {
    Rc::new(WatchedFiles {
        id: Cell::new(watcher_id_add(1)),
        name: name.to_string(),
        watch_kind,
        has_relative_pattern_capability,
        compute_glob_patterns,
        input: T::default(),
        compute_watchers_once: Cell::new(false),
        workspace_watchers: RefCell::new(Vec::new()),
        outside_workspace_watchers: RefCell::new(Vec::new()),
        ignored: RefCell::new(FxHashSet::default()),
    })
}

// Go: project/watch.go:186 NewWatchedFilesForPaths (tsgo#4712)
// NewWatchedFilesForPaths creates a watcher for exact file paths, routing files outside the workspace
// through directory-based external watchers so clients can use URI-based RelativePatterns when supported.

// Go: project/watch.go:211 Watchers
#[derive(Clone, Debug, Default)]
pub struct Watchers {
    pub watcher_id: WatcherID,
    pub workspace_watchers: Vec<lsproto::FileSystemWatcher>,
    pub outside_workspace_watchers: Vec<lsproto::FileSystemWatcher>,
    pub ignored_paths: FxHashSet<String>,
}

impl<T> WatchedFiles<T> {
    // Go: project/watch.go:218 WatchedFiles.Watchers
    // PORT: Go returns the shared slices and map; the port returns copies.

    // Go: project/watch.go:266 WatchedFiles.ID
    // PORT: Go allows a nil receiver; call as `WatchedFiles::id(w.as_deref())`.

    // Go: project/watch.go:273 WatchedFiles.Name
    pub fn name(&self) -> String {
        self.name.clone()
    }

    // Go: project/watch.go:277 WatchedFiles.WatchKind
    pub fn watch_kind(&self) -> lsproto::WatchKind {
        self.watch_kind
    }

    // Go: project/watch.go:281 WatchedFiles.Clone
    // PORT: Go allows a nil receiver; call as
    // `WatchedFiles::clone_(w.as_deref(), input)`. `clone_` keeps it apart
    // from `std::clone::Clone`. Go does not copy `id` (it stays 0) or
    // `ignored`.
    pub fn clone_(w: Option<&WatchedFiles<T>>, input: T) -> Option<Rc<WatchedFiles<T>>> {
        let w = w?;
        Some(Rc::new(WatchedFiles {
            name: w.name.clone(),
            watch_kind: w.watch_kind,
            has_relative_pattern_capability: w.has_relative_pattern_capability,
            compute_glob_patterns: w.compute_glob_patterns.clone(),
            workspace_watchers: RefCell::new(w.workspace_watchers.borrow().clone()),
            outside_workspace_watchers: RefCell::new(w.outside_workspace_watchers.borrow().clone()),
            input,
            compute_watchers_once: Cell::new(false),
            ignored: RefCell::new(FxHashSet::default()),
            id: Cell::new(0),
        }))
    }
}

// Go: project/watch.go:298 createResolutionLookupGlobMapper
// PORT: the Go input `*collections.SyncSet[tspath.Path]` is
// `Option<Rc<RefCell<FxHashSet<tspath::Path>>>>` (the sourceFS seen files).
// Go ranges over the set (random order); the result does not depend on the
// order (one file per directory, sorted outputs).

// Go: project/watch.go:380 getTypingsLocationsGlobs
// PORT: Go maps are `IndexMap`s (insertion order; Go map order is random).
// The globs are sorted later (`WatchedFiles.Watchers`) and the external
// directories go through `GetCommonParents` and a sort.

// Go: project/watch.go:424 getPathComponentsForWatching

// Go: project/watch.go:434 perceivedOsRootLengthForWatching

// Go: project/watch.go:458 getRecursiveGlobPattern
pub fn get_recursive_glob_pattern(directory: &str) -> String {
    format!(
        "{}/{}",
        tspath::remove_trailing_directory_separator(directory),
        "**/*"
    )
}

// Go: project/watch.go:464 recursiveDirectoryGlobPattern
// recursiveDirectoryGlobPattern returns the string form of a recursive watcher
// for the given directory that would be produced by newRecursiveDirectoryWatcher.

// Go: project/watch.go:474 newRecursiveDirectoryWatcher
// newRecursiveDirectoryWatcher creates a FileSystemWatcher for recursively
// watching a directory. When useRelativePattern is true, a RelativePattern with
// a file:// base URI is used; otherwise a plain glob Pattern is used.
// PORT: Go returns a pointer; the watcher lists hold values.
