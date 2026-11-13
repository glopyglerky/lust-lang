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
use crate::xxhash_rust::xxh3::xxh3_128;

// Go: project/snapshotfs.go:21 FileHandleSource (ts#64291)
pub trait FileHandleSource {
    fn get_file(&self, file_name: &str) -> Option<Rc<dyn FileHandle>>;
    fn get_file_by_path(&self, file_name: &str, path: &tspath::Path) -> Option<Rc<dyn FileHandle>>;
}

// Go: project/snapshotfs.go:26 FileSource
pub trait FileSource: FileHandleSource {
    fn fs(&self) -> Rc<dyn vfs::Fs>;
    fn file_exists(&self, file_name: &str, path: &tspath::Path) -> bool;
    fn get_accessible_entries(&self, path: &str) -> vfs::Entries;

    /// Go `source.FS().UseCaseSensitiveFileNames()`. A released source
    /// (`SourceFS::release`) answers it with no file system.
    // PORT: not in Go.
    fn use_case_sensitive_file_names(&self) -> bool {
        self.fs().use_case_sensitive_file_names()
    }
}

// Go: project/snapshotfs.go:33 cachedLayeredFileSystem (ts#64291)
// PORT: Go embeds `*cachedvfs.FS`; here it is the field `fs`, and the
// `vfs.FS` methods forward to it.
pub struct CachedLayeredFileSystem {
    pub fs: Rc<vfs::CachedFs>,
    pub layered: Rc<dyn LayeredFileSystem>,
}

// Go: project/snapshotfs.go:38 newCachedLayeredFileSystem
// PORT: the cache asks `layered` through an `AheadLookupLayer`.

/// The layer under a snapshot's lookup cache (`CachedLayeredFileSystem`)
/// that answers `file_exists`, `directory_exists` and `realpath` from the
/// lookups of the resolve-ahead jobs of the snapshot's program loads
/// (compiler/resolve_ahead.rs `AheadLookups`) before it asks the layered
/// file system. The workers made these calls during the loads, as Go's
/// parse tasks make theirs on the one cache of the snapshot, so the
/// snapshot has their answers with no copy. The job that was attached first
/// gives the answer.
// PORT: not in Go (perf).
pub struct AheadLookupLayer {
    layered: Rc<dyn LayeredFileSystem>,
    jobs: RefCell<Vec<Arc<dyn AheadLookups>>>,
}

impl AheadLookupLayer {
    /// Adds the lookups of a load's job (`ResolveAheadHost::attach`), and
    /// gives their index.
    pub fn attach(&self, lookups: Arc<dyn AheadLookups>) -> usize {
        let mut jobs = self.jobs.borrow_mut();
        jobs.push(lookups);
        jobs.len() - 1
    }

    /// The first answer of `get` in the jobs attached before job `index`.
    pub fn before<T>(
        &self,
        index: usize,
        get: impl Fn(&dyn AheadLookups) -> Option<T>,
    ) -> Option<T> {
        self.jobs
            .borrow()
            .iter()
            .take(index)
            .find_map(|lookups| get(&**lookups))
    }

    fn first<T>(&self, get: impl Fn(&dyn AheadLookups) -> Option<T>) -> Option<T> {
        self.before(usize::MAX, get)
    }
}


impl vfs::Fs for AheadLookupLayer {
    fn use_case_sensitive_file_names(&self) -> bool {
        self.layered.use_case_sensitive_file_names()
    }
    fn file_exists(&self, path: &str) -> bool {
        self.first(|lookups| lookups.file_exists(path))
            .unwrap_or_else(|| self.layered.file_exists(path))
    }
    fn read_file(&self, path: &str) -> (String, bool) {
        self.layered.read_file(path)
    }
    fn write_file(&self, path: &str, data: &str) -> Result<(), vfs::FsError> {
        self.layered.write_file(path, data)
    }
    fn append_file(&self, path: &str, data: &str) -> Result<(), vfs::FsError> {
        self.layered.append_file(path, data)
    }
    fn remove(&self, path: &str) -> Result<(), vfs::FsError> {
        self.layered.remove(path)
    }
    fn chtimes(
        &self,
        path: &str,
        a_time: Option<SystemTime>,
        m_time: Option<SystemTime>,
    ) -> Result<(), vfs::FsError> {
        self.layered.chtimes(path, a_time, m_time)
    }
    fn directory_exists(&self, path: &str) -> bool {
        self.first(|lookups| lookups.directory_exists(path))
            .unwrap_or_else(|| self.layered.directory_exists(path))
    }
    fn get_accessible_entries(&self, path: &str) -> vfs::Entries {
        self.layered.get_accessible_entries(path)
    }
    fn stat(&self, path: &str) -> Option<vfs::FileInfo> {
        self.layered.stat(path)
    }
    fn realpath(&self, path: &str) -> String {
        self.first(|lookups| lookups.realpath(path))
            .unwrap_or_else(|| self.layered.realpath(path))
    }
    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
}

/// The `AheadLookupLayer` under the lookup cache `cached` of a snapshot
/// (`new_cached_layered_file_system`).
pub fn ahead_lookup_layer(cached: &vfs::CachedFs) -> Option<&AheadLookupLayer> {
    vfs::Fs::as_any(&**cached.wrapped())?.downcast_ref::<AheadLookupLayer>()
}

impl FileHandleSource for CachedLayeredFileSystem {
    // Go: project/snapshotfs.go:45 cachedLayeredFileSystem.GetFile
    fn get_file(&self, file_name: &str) -> Option<Rc<dyn FileHandle>> {
        self.layered.get_file(file_name)
    }

    // Go: project/snapshotfs.go:49 cachedLayeredFileSystem.GetFileByPath
    fn get_file_by_path(&self, file_name: &str, path: &tspath::Path) -> Option<Rc<dyn FileHandle>> {
        self.layered.get_file_by_path(file_name, path)
    }
}




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
pub struct SnapshotFS {
    pub to_path: Rc<dyn Fn(&str) -> tspath::Path>,
    pub fs: Rc<dyn LayeredFileSystem>,
    pub cache_files: Rc<FxHashMap<tspath::Path, Rc<RefCell<CachedFile>>>>,
    pub cache_directories: Rc<FxHashMap<tspath::Path, CachedDirectory>>,
    pub read_files: RefCell<FxHashMap<tspath::Path, MemoizedCachedFile>>,
    // nodeModulesRealpathAliases maps realpath-based keys to sets of symlink-based keys,
    // for files inside node_modules that are accessed through directory symlinks.
    // This allows watch events (which use realpaths) to invalidate files cached under symlink paths.
    pub node_modules_realpath_aliases: Rc<FxHashMap<tspath::Path, Rc<RefCell<RealpathAliasSet>>>>,
}

// Go: project/snapshotfs.go:104 memoizedCachedFile
// PORT: Go `func() FileHandle` made by `sync.OnceValue`; the closure keeps
// its value in a `OnceCell`.
pub type MemoizedCachedFile = Rc<dyn Fn() -> Option<Rc<dyn FileHandle>>>;

impl SnapshotFS {
    // Go: project/snapshotfs.go:578 SnapshotFS.expandRealpathAliases
    // expandRealpathAliases adds synthetic URIs to the Changed and Deleted sets for
    // files that were accessed through node_modules symlinks. When a watch event arrives
    // using a realpath, this expands it to include the symlink-based path so that
    // downstream consumers (markDirtyFiles, markFilesChanged) can find cached entries.
}

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
pub struct SnapshotFSBuilder {
    pub fs: Rc<dyn LayeredFileSystem>,
    pub cache_files: Rc<dirty::SyncMap<tspath::Path, Rc<RefCell<CachedFile>>>>,
    pub cache_directories: Rc<dirty::Map<tspath::Path, CachedDirectory>>,
    pub source_backed_replacements: RefCell<FxHashSet<tspath::Path>>,
    pub node_modules_realpath_aliases:
        Rc<dirty::SyncMap<tspath::Path, Rc<RefCell<RealpathAliasSet>>>>,
    pub to_path: Rc<dyn Fn(&str) -> tspath::Path>,
}

// Go: project/snapshotfs.go:186 newSnapshotFSBuilderFromSource (ts#64291)
// PORT: the base maps are shared `Rc` maps, as Go shares its maps (no
// copy).

// Go: project/snapshotfs.go:238 onDeletedFileOrDirectory (a closure in Finalize)
// PORT: the recursive Go closure is a nested function over the map it reads.

impl SnapshotFSBuilder {
    // Go: project/snapshotfs.go:208 snapshotFSBuilder.Finalize

    // Go: project/snapshotfs.go:305 snapshotFSBuilder.deleteCacheEntry (ts#64291)

    // Go: project/snapshotfs.go:348 snapshotFSBuilder.cacheSourceFile (ts#64291)

    // Go: project/snapshotfs.go:361 snapshotFSBuilder.getCachedFile (ts#64291: was getDiskFile)

    // Go: project/snapshotfs.go:378 snapshotFSBuilder.recordRealpathAlias
    // recordRealpathAlias checks if fileName is accessed through a symlink and, if so,
    // records a mapping from the realpath-based key to the symlink-based key.
    // This is only called for files inside node_modules where symlinks are common.

    // Go: project/snapshotfs.go:392 snapshotFSBuilder.reloadEntry

    // Go: project/snapshotfs.go:424 snapshotFSBuilder.reloadEntryIfNeeded

    // Go: project/snapshotfs.go:455 snapshotFSBuilder.watchChangesOverlapCache
    // PORT: Go passes the summary by value; here by reference.

    // Go: project/snapshotfs.go:483 snapshotFSBuilder.invalidateCache

    // Go: project/snapshotfs.go:492 snapshotFSBuilder.invalidateNodeModulesCache

    // Go: project/snapshotfs.go:503 snapshotFSBuilder.markDirtyFiles
    // PORT: Go reloads the changed disk files in a work group; the port
    // reloads them one after another (one thread). The result is a set, so
    // the order does not matter.

    // Go: project/snapshotfs.go:540 snapshotFSBuilder.reloadEntryIfContentChanged
    // PORT: Go named result `(changed bool)`.

    // Go: project/snapshotfs.go:615 snapshotFSBuilder.isRelevantFileName
    // isRelevantFileName returns true if the given URI refers to a file that
    // could affect the project: it has a TypeScript-relevant or configured content-mapper extension,
    // is dynamic (e.g. untitled), or is present in the supplied open-file state.
    // PORT: tsgo#4712 adds the content mapper arguments. Go nil
    // `*collections.Set` is `None`.

    // Go: project/snapshotfs.go:651 snapshotFSBuilder.expandAndFilterWatchEvents
    // expandAndFilterWatchEvents expands directory deletion URIs into individual
    // file deletion URIs using the cached directory structure, and filters out
    // watch events for paths that are neither known directories nor have relevant
    // file extensions.

    // Go: project/snapshotfs.go:709 snapshotFSBuilder.collectFilesRecursive
    // collectFilesRecursive recursively collects all cached file URIs under the
    // given directory path using the cacheDirectories and cacheFiles maps.

    // Go: project/snapshotfs.go:734 snapshotFSBuilder.convertOpenAndCloseToChanges
}

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
impl SnapshotFSBuilder {



    /// The cache of this snapshot's file system lookups (Go
    /// `cachedLayeredFileSystem`'s `*cachedvfs.FS`), over the layer that
    /// takes the answers of the resolve-ahead jobs (`ahead_lookup_layer`).
    pub fn cached_fs(&self) -> Option<Rc<vfs::CachedFs>> {
        Some(self.cached_layered()?.fs.clone())
    }

    fn cached_layered(&self) -> Option<&CachedLayeredFileSystem> {
        vfs::Fs::as_any(&*self.fs)?.downcast_ref::<CachedLayeredFileSystem>()
    }

}

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
pub struct SourceFS {
    pub tracking: Cell<bool>,
    pub to_path: Rc<dyn Fn(&str) -> tspath::Path>,
    pub missing_directories: Option<Rc<RefCell<FxHashSet<tspath::Path>>>>,
    pub seen_files: RefCell<Option<Rc<RefCell<FxHashSet<tspath::Path>>>>>,
    pub source: RefCell<Rc<dyn FileSource>>,
}

// Go: project/snapshotfs.go:777 newSourceFS
pub fn new_source_fs(
    tracking: bool,
    source: Rc<dyn FileSource>,
    to_path: Rc<dyn Fn(&str) -> tspath::Path>,
) -> Rc<SourceFS> {
    let mut fs = SourceFS {
        tracking: Cell::new(tracking),
        to_path,
        missing_directories: None,
        seen_files: RefCell::new(None),
        source: RefCell::new(source),
    };
    if tracking {
        fs.seen_files = RefCell::new(Some(Rc::new(RefCell::new(FxHashSet::default()))));
        fs.missing_directories = Some(Rc::new(RefCell::new(FxHashSet::default())));
    }
    Rc::new(fs)
}

impl SourceFS {
    /// Go `fs.source` (read). The handle is copied so no borrow is held
    /// while the source runs.
    fn source(&self) -> Rc<dyn FileSource> {
        self.source.borrow().clone()
    }

    // Go: project/snapshotfs.go:792 sourceFS.DisableTracking
    pub fn disable_tracking(&self) {
        self.tracking.set(false);
    }


    // Go: project/snapshotfs.go:796 sourceFS.Track
    pub fn track(&self, file_name: &str) {
        if !self.tracking.get() {
            return;
        }
        let path = (self.to_path)(file_name);
        self.seen_files
            .borrow()
            .as_ref()
            .unwrap_or_else(|| crate::core::go_nil_dereference())
            .borrow_mut()
            .insert(path);
    }

    // Go: project/snapshotfs.go:803 sourceFS.SeenFile
    pub fn seen_file(&self, path: &tspath::Path) -> bool {
        let seen_files = self.seen_files.borrow();
        let Some(seen_files) = seen_files.as_ref() else {
            return false;
        };
        let seen = seen_files.borrow().contains(path);
        seen
    }

    // Go: project/snapshotfs.go:810 sourceFS.SeenFileOrMissingParentDirectory

    // Go: project/snapshotfs.go:830 sourceFS.GetFile
    pub fn get_file(&self, file_name: &str) -> Option<Rc<dyn FileHandle>> {
        self.track(file_name);
        self.source().get_file(file_name)
    }

    // Go: project/snapshotfs.go:835 sourceFS.GetFileByPath
    pub fn get_file_by_path(
        &self,
        file_name: &str,
        path: &tspath::Path,
    ) -> Option<Rc<dyn FileHandle>> {
        self.track(file_name);
        self.source().get_file_by_path(file_name, path)
    }

    /// Drops the file source and the tracked sets when the host of this file
    /// system is released (`compiler::CompilerHost::release`). Only the case
    /// sensitivity stays, so a file name lookup on a released program still
    /// works (Go `Program.GetSourceFile` makes a path first). Any file
    /// access panics after this. `seen_files` can be shared with the host of
    /// a program cloned from this host's program; only this reference goes.
    // PORT: not in Go. Go's GC frees the `sourceFS` with its host.
    pub fn release(&self) {
        self.tracking.set(false);
        let released: Rc<dyn FileSource> = Rc::new(ReleasedFileSource {
            use_case_sensitive_file_names: self.source().use_case_sensitive_file_names(),
        });
        // PORT: the old values drop after the borrows end.
        let source = std::mem::replace(&mut *self.source.borrow_mut(), released);
        let seen_files = self.seen_files.borrow_mut().take();
        let missing_directories = self
            .missing_directories
            .as_ref()
            .map(|missing| std::mem::take(&mut *missing.borrow_mut()));
        drop(source);
        drop(seen_files);
        drop(missing_directories);
    }
}

// PORT: not in Go. The replay of a resolve-ahead answer's calls
// (project/compilerhost.rs): the side effects of `vfs::Fs::file_exists`
// and `vfs::Fs::directory_exists` with the path made already.
impl SourceFS {
    /// `track` of a name whose path is `path`.
    pub fn track_path(&self, path: &tspath::Path) {
        if !self.tracking.get() {
            return;
        }
        let seen_files = self.seen_files.borrow();
        let mut seen_files = seen_files
            .as_ref()
            .unwrap_or_else(|| crate::core::go_nil_dereference())
            .borrow_mut();
        if !seen_files.contains(path) {
            seen_files.insert(path.clone());
        }
    }

    /// What `directory_exists` notes when it gives false for a name whose
    /// path is `path`.
    pub fn note_missing_directory(&self, path: &tspath::Path) {
        if !self.tracking.get() {
            return;
        }
        let mut missing_directories = self
            .missing_directories
            .as_ref()
            .unwrap_or_else(|| crate::core::go_nil_dereference())
            .borrow_mut();
        if !missing_directories.contains(path) {
            missing_directories.insert(path.clone());
        }
    }
}

/// The file source of a released `SourceFS` (`SourceFS::release`). It keeps
/// only the case sensitivity; every file access panics.
// PORT: not in Go.
struct ReleasedFileSource {
    use_case_sensitive_file_names: bool,
}

fn released_file_source_used() -> ! {
    panic!("the file system of a released program's compiler host was used");
}

impl FileHandleSource for ReleasedFileSource {
    fn get_file(&self, _file_name: &str) -> Option<Rc<dyn FileHandle>> {
        released_file_source_used()
    }

    fn get_file_by_path(
        &self,
        _file_name: &str,
        _path: &tspath::Path,
    ) -> Option<Rc<dyn FileHandle>> {
        released_file_source_used()
    }
}

impl FileSource for ReleasedFileSource {
    fn fs(&self) -> Rc<dyn vfs::Fs> {
        released_file_source_used()
    }

    fn file_exists(&self, _file_name: &str, _path: &tspath::Path) -> bool {
        released_file_source_used()
    }

    fn get_accessible_entries(&self, _path: &str) -> vfs::Entries {
        released_file_source_used()
    }

    fn use_case_sensitive_file_names(&self) -> bool {
        self.use_case_sensitive_file_names
    }
}

// Go: project/snapshotfs.go:664 `var _ vfs.FS = (*sourceFS)(nil)`
impl vfs::Fs for SourceFS {
    // Go: project/snapshotfs.go:879 sourceFS.UseCaseSensitiveFileNames
    // UseCaseSensitiveFileNames implements vfs.FS.
    fn use_case_sensitive_file_names(&self) -> bool {
        // PORT: through the source, so a released source can answer it
        // (`SourceFS::release`).
        self.source().use_case_sensitive_file_names()
    }

    // Go: project/snapshotfs.go:850 sourceFS.FileExists
    // FileExists implements vfs.FS.
    fn file_exists(&self, path: &str) -> bool {
        // PORT: Go makes the path twice (`Track` and the source call); the
        // port makes it once, with the same `to_path`.
        let file_path = (self.to_path)(path);
        self.track_path(&file_path);
        self.source().file_exists(path, &file_path)
    }

    // Go: project/snapshotfs.go:861 sourceFS.ReadFile
    // ReadFile implements vfs.FS.
    fn read_file(&self, path: &str) -> (String, bool) {
        if let Some(fh) = self.get_file(path) {
            return (fh.content(), true);
        }
        (String::new(), false)
    }

    // Go: project/snapshotfs.go:884 sourceFS.WriteFile
    // WriteFile implements vfs.FS.
    fn write_file(&self, _path: &str, _data: &str) -> Result<(), vfs::FsError> {
        crate::core::go_panic("unimplemented".to_string());
    }

    // Go: project/snapshotfs.go:889 sourceFS.AppendFile
    // AppendFile implements vfs.FS.
    fn append_file(&self, _path: &str, _data: &str) -> Result<(), vfs::FsError> {
        crate::core::go_panic("unimplemented".to_string());
    }

    // Go: project/snapshotfs.go:894 sourceFS.Remove
    // Remove implements vfs.FS.
    fn remove(&self, _path: &str) -> Result<(), vfs::FsError> {
        crate::core::go_panic("unimplemented".to_string());
    }

    // Go: project/snapshotfs.go:899 sourceFS.Chtimes
    // Chtimes implements vfs.FS.
    fn chtimes(
        &self,
        _path: &str,
        _a_time: Option<SystemTime>,
        _m_time: Option<SystemTime>,
    ) -> Result<(), vfs::FsError> {
        crate::core::go_panic("unimplemented".to_string());
    }

    // Go: project/snapshotfs.go:841 sourceFS.DirectoryExists
    // DirectoryExists implements vfs.FS.
    fn directory_exists(&self, path: &str) -> bool {
        let exists = self.source().fs().directory_exists(path);
        if !exists && self.tracking.get() {
            self.missing_directories
                .as_ref()
                .unwrap_or_else(|| crate::core::go_nil_dereference())
                .borrow_mut()
                .insert((self.to_path)(path));
        }
        exists
    }

    // Go: project/snapshotfs.go:856 sourceFS.GetAccessibleEntries
    // GetAccessibleEntries implements vfs.FS.
    fn get_accessible_entries(&self, path: &str) -> vfs::Entries {
        self.source().get_accessible_entries(path)
    }

    // Go: project/snapshotfs.go:874 sourceFS.Stat
    // Stat implements vfs.FS.
    fn stat(&self, path: &str) -> Option<vfs::FileInfo> {
        self.source().fs().stat(path)
    }

    // Go: project/snapshotfs.go:869 sourceFS.Realpath
    // Realpath implements vfs.FS.
    fn realpath(&self, path: &str) -> String {
        self.source().fs().realpath(path)
    }
}
