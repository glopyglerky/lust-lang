//! Go `internal/project/snapshotfs.go`.
//!
//! PORT: one thread (project/dirty/interfaces.rs). Go mutexes are dropped.
//! `collections.SyncMap` / `SyncSet` are `RefCell<FxHashMap>` /
//! `RefCell<FxHashSet>`; `collections.Set` is `FxHashSet`. A shared Go
//! `*collections.SyncSet` is `Rc<RefCell<FxHashSet>>` (nil is `None`). Go
//! `func(fileName string) tspath.Path` is `Rc<dyn Fn(&str) -> tspath::Path>`.
//! Overlay maps are `IndexMap` (see overlayfs.rs). Go `xxh3.Uint128` is
//! `u128`.

use crate::project::prelude::*;

use crate::frontend::compiler::resolve_ahead::AheadLookups;
use std::cell::{Cell, OnceCell};
use std::sync::Arc;
use std::time::SystemTime;

// Go: project/snapshotfs.go:21 FileHandleSource (ts#64291)

// Go: project/snapshotfs.go:26 FileSource

// Go: project/snapshotfs.go:33 cachedLayeredFileSystem (ts#64291)
// PORT: Go embeds `*cachedvfs.FS`; here it is the field `fs`, and the
// `vfs.FS` methods forward to it.

// Go: project/snapshotfs.go:38 newCachedLayeredFileSystem
// PORT: the cache asks `layered` through an `AheadLookupLayer`.










// PORT: the promoted `*cachedvfs.FS` methods.

// Go: project/snapshotfs.go:71 realpathAliasSet
// realpathAliasSet is a thread-safe set of symlink paths that alias a single realpath.
// It implements dirty.Cloneable so it can be used as a value in dirty.SyncMap.
// PORT: `mu` is dropped. Go `*realpathAliasSet` is `Rc<RefCell<..>>`.
#[derive(Debug, Default)]
pub struct RealpathAliasSet {
    pub paths: FxHashSet<tspath::Path>,
}

impl RealpathAliasSet {
    // Go: project/snapshotfs.go:76 realpathAliasSet.Add
    pub fn add(&mut self, path: tspath::Path) {
        self.paths.insert(path);
    }

    // Go: project/snapshotfs.go:82 realpathAliasSet.Clone
    // PORT: Go `Clone`; `clone_` keeps it apart from `std::clone::Clone`.
    pub fn clone_(&self) -> Rc<RefCell<RealpathAliasSet>> {
        let mut clone = RealpathAliasSet::default();
        if !self.paths.is_empty() {
            clone.paths = self.paths.clone();
        }
        Rc::new(RefCell::new(clone))
    }
}

impl dirty::Cloneable for Rc<RefCell<RealpathAliasSet>> {
    fn clone_(&self) -> Self {
        self.borrow().clone_()
    }
}

// Go: project/snapshotfs.go:92 SnapshotFS
// PORT: Go shares `cacheFiles`, `cacheDirectories` and
// `nodeModulesRealpathAliases` between snapshots until one of them changes
// (a Go map is a reference). They are `Rc` maps here, so a snapshot clone
// with no cache change copies no map, and dropping an old snapshot frees
// nothing that the next one still uses.

// Go: project/snapshotfs.go:104 memoizedCachedFile
// PORT: Go `func() FileHandle` made by `sync.OnceValue`; the closure keeps
// its value in a `OnceCell`.


// Go: project/snapshotfs.go:66 `_ FileSource = (*SnapshotFS)(nil)`


/// Go `dirty.CloneableMap[tspath.Path, string]` of `cacheDirectories`: the
/// cached children of one directory, by path, with their base names.
// PORT: Go ranges over it (`mergeCachedDirectoryEntries`). A Go map of at
// most 8 entries is one group of 8 slots: an insert takes the first free
// slot, a delete frees its slot, and a range starts at a random slot and
// wraps. `dirty.CloneableMap.Clone` (`maps.Clone`) keeps the slots. It is an
// `IndexMap` (insertion order). With no delete, the port gives the rotation
// that starts at the first slot, which Go can give. A delete keeps the order
// of the other children (`shift_remove`), as Go keeps their slots. Go can
// differ in two cases:
// - A delete and then an add: Go puts the new child in the first free slot,
//   which can be the slot of the deleted child, but the port puts it last.
// - Above 8 children: Go's order depends on the hash seed of the map.
// The FxHashMap of `dirty::CloneableMap` gave the order of the path hashes,
// which Go can miss (editfuzz2 R1, as for `OverlayDirectories`).
pub type CachedDirectory = Rc<RefCell<IndexMap<tspath::Path, String>>>;

impl dirty::Cloneable for CachedDirectory {
    // Go: project/dirty/cloneablemap.go:7 Clone
    fn clone_(&self) -> Self {
        Rc::new(RefCell::new(self.borrow().clone()))
    }
}

// Go: project/snapshotfs.go:144 mergeCachedDirectoryEntries (ts#64291)
// PORT: Go ranges over the cached entries map (random order); insertion
// order here (`CachedDirectory`).

// Go: project/snapshotfs.go:177 snapshotFSBuilder

// Go: project/snapshotfs.go:186 newSnapshotFSBuilderFromSource (ts#64291)
// PORT: the base maps are shared `Rc` maps, as Go shares its maps (no
// copy).

// Go: project/snapshotfs.go:238 onDeletedFileOrDirectory (a closure in Finalize)
// PORT: the recursive Go closure is a nested function over the map it reads.


// Go: project/snapshotfs.go:65 `_ FileSource = (*snapshotFSBuilder)(nil)`


/// The state of a `cache_files` entry that `FileSource::file_exists`
/// reads (`SnapshotFSBuilder::cached_file_state`).
// PORT: not in Go (resolve ahead, compiler/resolve_ahead.rs).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CachedFileState {
    /// No entry: `file_exists` asks the layered file system.
    Absent,
    /// A live entry that needs no reload: `file_exists` is true.
    Live,
    /// An entry with no value (a deleted file): `file_exists` is false.
    NoValue,
    /// An entry that `file_exists` would reload first (a read).
    NeedsReload,
}

// PORT: not in Go. What resolve ahead needs to check a worker's answer
// on the snapshot file system (project/compilerhost.rs).

// Go: project/snapshotfs.go:639 isRelevantExtension
// isRelevantExtension returns true if the given extension is a known TypeScript
// or JavaScript extension that can affect the project.
pub fn is_relevant_extension(ext: &str) -> bool {
    matches!(
        ext,
        ".js" | ".jsx" | ".mjs" | ".cjs" | ".ts" | ".tsx" | ".mts" | ".cts" | ".json"
    )
}

// Go: project/snapshotfs.go:688 isNodeModulesPath
// isNodeModulesPath reports whether path is a node_modules directory itself or
// lives inside one. Used to preserve node_modules watch deletions, whose package
// files are read transiently and therefore never tracked in cacheDirectories.
pub fn is_node_modules_path(path: &tspath::Path) -> bool {
    let s = path.as_str();
    s.ends_with("/node_modules") || s.contains("/node_modules/")
}

// Go: project/snapshotfs.go:693 hasOpenFileWithin (ts#64291)

// Go: project/snapshotfs.go:769 sourceFS
// sourceFS is a vfs.FS that sources files from a FileSource and tracks seen files.
// PORT: Go `*sourceFS` is shared (`Rc<SourceFS>`, also as `Rc<dyn vfs::Fs>`).
// Go writes `tracking`, `seenFiles` and `source` after sharing, so they are
// `Cell` / `RefCell`.

// Go: project/snapshotfs.go:777 newSourceFS


// PORT: not in Go. The replay of a resolve-ahead answer's calls
// (project/compilerhost.rs): the side effects of `vfs::Fs::file_exists`
// and `vfs::Fs::directory_exists` with the path made already.

/// The file source of a released `SourceFS` (`SourceFS::release`). It keeps
/// only the case sensitivity; every file access panics.
// PORT: not in Go.
struct ReleasedFileSource {
    use_case_sensitive_file_names: bool,
}

fn released_file_source_used() -> ! {
    panic!("the file system of a released program's compiler host was used");
}



// Go: project/snapshotfs.go:664 `var _ vfs.FS = (*sourceFS)(nil)`
