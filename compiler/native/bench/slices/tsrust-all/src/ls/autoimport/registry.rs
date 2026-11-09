use crate::ls::autoimport::prelude::*;

// Port of Go `ls/autoimport/registry.go`.
//
// PORT (whole file):
// - Concurrency. `sync.WaitGroup` / `wg.Go` run each task at its start
//   point, serially in Go start order (map-project decision 1); the
//   `ctx.Err()` checks stay. Each task runs under
//   `core::go_wait_group_goroutine`, so a Go panic in it ends the process
//   as Go's goroutine does, and the request's recover does not catch it. A
//   build that the caller drops on cancel has more stop points
//   (`should_stop_build`). Mutexes are dropped. `atomic.Int32` is `Cell`.
// - Go map iteration that feeds the index (`exports` maps, the second-pass
//   root files and lookup sources) uses `IndexMap` / `IndexSet` in insertion
//   order. PORT: Go map order is random. Index order decides ties in the
//   completion sort, and also which path a merged completion export keeps,
//   which can change module specifiers through the specifier cache (see
//   `View::get_completions` in `view.rs`). Other Go maps are `FxHashMap` /
//   `FxHashSet`; their order only reaches logs or sets.
// - Pointers. `*Registry`, `*RegistryBucket`, `*Export`,
//   `*module.ResolvedEntrypoint` are `Rc`. `*directory` is
//   `Rc<RefCell<Directory>>` (mutated through `dirty.Map` changes). A bucket
//   changes after it is shared only in `state` (a `RefCell`) and in its
//   index during the second pass (`Rc<RefCell<Index>>`). Go `Clone()` is
//   `clone_()`.
// - Go `*collections.Set[string]` is `Option<FxHashSet<String>>` (`None` is
//   nil; the nil-safe Go methods are the `set_*` helpers below). Buckets hold
//   shared sets and maps behind `Rc`, as Go shares the pointers.
// - Go `*packagejson.InfoCacheEntry` is `Option<Rc<packagejson::InfoCacheEntry>>`;
//   a Go dereference of nil panics with the Go runtime text.
// - Go `*logging.LogTree` is `Option<Rc<logging::LogTree>>`. Log text uses
//   `{:?}` for Go `%v` durations (log text is not compared).
// - Go `*ast.SourceFile` from `program.GetSourceFiles()` is `&ParsedSourceFile`
//   where only `FileName()` and `Path()` are read, and `file.root` where a
//   node is needed.

use crate::flags_macros::go_enum;
use crate::frontend::compiler;
use crate::frontend::core_ext::HasFileName;
use crate::frontend::core_ls_ext::{diff_maps_func, unordered_equal};
use crate::frontend::module;
use crate::frontend::packagejson;
use crate::frontend::tspath;
use crate::frontend::vfs;
use crate::gostd::context::ContextKey;
use crate::gostd::{GoError};
use crate::ls::{lsconv, lsutil};
use crate::lsp::lsproto;
use crate::program::ls_program;
use crate::project::logging::{LogTreeMethods as _, Logger as _};
use crate::project::{dirty, logging};
use std::sync::LazyLock;
use std::time::Instant;

// Go: ls/autoimport/registry.go:32 ProjectID
// PORT: Go `ProjectID` is an interface (`fmt.Stringer`) that the project
// package fills with its string type `project.ID`; interface map keys compare
// by that value. Every value is a `project.ID`, so the Rust type holds its
// string: the project package passes `ProjectID(id.to_string())`. A Go nil
// `ProjectID` is `None`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProjectID(pub String);

impl ProjectID {
    /// Go `fmt.Stringer.String`.
    #[must_use]
    pub fn string(&self) -> String {
        self.0.clone()
    }
}

/// Go `%s` / `%v` of a `ProjectID` is its `String()`.
impl std::fmt::Display for ProjectID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

// Go: ls/autoimport/registry.go:36 knownRecursiveSearchPackages
pub static KNOWN_RECURSIVE_SEARCH_PACKAGES: LazyLock<FxHashSet<&'static str>> =
    LazyLock::new(|| {
        [
            "@material-ui/core",
            "@material-ui/icons",
            "@sap/cds",
            "@testing-library/react-native",
            "ajv",
            "asap",
            "async",
            "aws-sdk",
            "braintree-web",
            "core-js",
            "core-js-pure",
            "crypto-js",
            "cypress-mochawesome-reporter",
            "dd-trace",
            "dumi",
            "dva",
            "egg-mock",
            "electron-log",
            "es-abstract",
            "es6-promise",
            "eslint-config-taro",
            "expo",
            "expo-router",
            "flow-remove-types",
            "gatsby",
            "glamor",
            "gluegun",
            "graphology-indices",
            "graphology-traversal",
            "graphology-utils",
            "jest-expo",
            "lodash",
            "lodash-es",
            "moment",
            "mz",
            "next",
            "pdfjs-dist",
            "protobufjs",
            "react-app-polyfill",
            "react-dev-utils",
            "react-devtools-inline",
            "recast",
            "semver",
            "stylelint-config-html",
            "umi",
            "web3-provider-engine",
            "webpack",
        ]
        .into_iter()
        .collect()
    });

// Go: ls/autoimport/registry.go:86 newProgramStructure
go_enum!(NewProgramStructure, i32 {
    FALSE = 0; // newProgramStructureFalse
    SAME_FILE_NAMES = 1; // newProgramStructureSameFileNames
    DIFFERENT_FILE_NAMES = 2; // newProgramStructureDifferentFileNames
});

// ---------------------------------------------------------------------------
// Go `collections.Set` methods on a possibly nil set.
// ---------------------------------------------------------------------------

// Go: collections/set.go:16 Has
fn set_has(s: Option<&FxHashSet<String>>, key: &str) -> bool {
    match s {
        None => false,
        Some(s) => s.contains(key),
    }
}

// Go: collections/set.go:35 Len
fn set_len(s: Option<&FxHashSet<String>>) -> usize {
    s.map_or(0, |s| s.len())
}

// Go: collections/set.go:104 Equals
// PORT: Go first compares the pointers; equal pointers hold equal sets.
fn set_equals(s: Option<&FxHashSet<String>>, other: Option<&FxHashSet<String>>) -> bool {
    match (s, other) {
        (None, None) => true,
        (Some(s), Some(other)) => s == other,
        _ => false,
    }
}

// Go: collections/set.go:114 IsSubsetOf
fn set_is_subset_of(s: Option<&FxHashSet<String>>, other: Option<&FxHashSet<String>>) -> bool {
    let Some(s) = s else {
        return true;
    };
    for key in s {
        if !set_has(other, key) {
            return false;
        }
    }
    true
}


// Go: ls/autoimport/registry.go:98 bucketBuildPreferences
// bucketBuildPreferences holds user preferences that affect how a bucket is
// built. When any of these change between builds, the bucket must be rebuilt.
// Adding a new preference here automatically integrates it into the rebuild
// checks via Equal.
#[derive(Clone, Debug, Default)]
pub struct BucketBuildPreferences {
    pub file_exclude_patterns: Vec<String>,
    pub auto_import_entrypoint_directory_search: Tristate,
}

// Go: ls/autoimport/registry.go:103 bucketBuildPreferencesFromUserPreferences
pub fn bucket_build_preferences_from_user_preferences(
    prefs: &lsutil::UserPreferences,
) -> BucketBuildPreferences {
    BucketBuildPreferences {
        file_exclude_patterns: prefs.auto_import_file_exclude_patterns.clone(),
        auto_import_entrypoint_directory_search: prefs.auto_import_entrypoint_directory_search,
    }
}

impl BucketBuildPreferences {
    // Go: ls/autoimport/registry.go:110 Equal
    pub fn equal(&self, other: &BucketBuildPreferences) -> bool {
        unordered_equal(&self.file_exclude_patterns, &other.file_exclude_patterns)
            && self.auto_import_entrypoint_directory_search
                == other.auto_import_entrypoint_directory_search
    }

    // Go: ls/autoimport/registry.go:110 Clone
    pub fn clone_(&self) -> BucketBuildPreferences {
        let mut p = self.clone();
        p.file_exclude_patterns = self.file_exclude_patterns.clone();
        p
    }
}

// Go: ls/autoimport/registry.go:129 BucketState
// BucketState represents the dirty state of a bucket.
// In general, a bucket can be used for an auto-imports request if it is clean
// or if the only edited file is the one that was requested for auto-imports.
// Most edits within a file will not change the imports available to that file.
// However, one exception causes the bucket to be rebuilt after a change to a
// single file: local files are newly added to the project by a manual import.
// This can only happen after a full (non-clone) program update. When this
// happens, the `newProgramStructure` flag is set until the next time the bucket
// is rebuilt, when this condition will be checked.
// PORT: the Go methods `DirtyFile`, `DirtyPackages` and
// `RecursiveSearchPackages` have the snake names of fields of the same type,
// so they end in `_exported` (PORTING "Names").
#[derive(Clone, Debug, Default)]
pub struct BucketState {
    // dirtyFile is the file that was edited last, if any. It does not necessarily
    // indicate that no other files have been edited, so it should be ignored if
    // `multipleFilesDirty` is set. It should not be used for node_modules buckets,
    // which rely on `dirtyPackages` instead.
    pub dirty_file: tspath::Path,
    pub multiple_files_dirty: bool,
    pub new_program_structure: NewProgramStructure,
    // buildPreferences holds the user preferences that were in effect when
    // the bucket was built. If changed, the bucket should be rebuilt.
    pub build_preferences: BucketBuildPreferences,
    // dirtyPackages is the set of package names that need to be re-indexed.
    // This is used for granular updates: when a file in a local workspace package
    // changes, only that package needs to be re-extracted rather than rebuilding
    // the entire node_modules bucket.
    // If nil, no granular updates are pending.
    // If set but multipleFilesDirty is true, the entire bucket needs to be rebuilt.
    pub dirty_packages: Option<FxHashSet<String>>,
    // recursiveSearchPackages tracks which packages were recursively directory-searched
    // when the bucket was built. nil means all non-exports packages were searched
    // (e.g. when the autoImportEntrypointDirectorySearch preference is enabled).
    // A non-nil set lists only the specific packages that were searched.
    // Used for rebuild detection: a rebuild is triggered when target packages are
    // not a subset of the currently searched packages.
    pub recursive_search_packages: Option<FxHashSet<String>>,
}

impl BucketState {
    // Go: ls/autoimport/registry.go:151 Clone
    pub fn clone_(&self) -> BucketState {
        let mut b = self.clone();
        b.build_preferences = self.build_preferences.clone_();
        b.dirty_packages = self.dirty_packages.clone();
        b.recursive_search_packages = self.recursive_search_packages.clone();
        b
    }

    // Go: ls/autoimport/registry.go:163 Dirty
    pub fn dirty(&self) -> bool {
        self.multiple_files_dirty
            || !self.dirty_file.is_empty()
            || self.new_program_structure.0 > 0
            || set_len(self.dirty_packages.as_ref()) > 0
    }

    // Go: ls/autoimport/registry.go:167 DirtyFile
    pub fn dirty_file_exported(&self) -> tspath::Path {
        if self.multiple_files_dirty {
            return tspath::Path::default();
        }
        self.dirty_file.clone()
    }

    // Go: ls/autoimport/registry.go:174 DirtyPackages
    pub fn dirty_packages_exported(&self) -> Option<&FxHashSet<String>> {
        if self.multiple_files_dirty {
            return None;
        }
        self.dirty_packages.as_ref()
    }

    // Go: ls/autoimport/registry.go:181 RecursiveSearchPackages
    pub fn recursive_search_packages_exported(&self) -> Option<&FxHashSet<String>> {
        self.recursive_search_packages.as_ref()
    }

    // Go: ls/autoimport/registry.go:185 possiblyNeedsRebuildForFile
    pub fn possibly_needs_rebuild_for_file(
        &self,
        file: &tspath::Path,
        preferences: &lsutil::UserPreferences,
    ) -> bool {
        self.new_program_structure.0 > 0
            || self.has_dirty_file_besides(file)
            || !self
                .build_preferences
                .equal(&bucket_build_preferences_from_user_preferences(preferences))
            || set_len(self.dirty_packages.as_ref()) > 0
    }

    // Go: ls/autoimport/registry.go:192 hasDirtyFileBesides
    pub fn has_dirty_file_besides(&self, file: &tspath::Path) -> bool {
        self.multiple_files_dirty || !self.dirty_file.is_empty() && self.dirty_file != *file
    }
}

// Go: ls/autoimport/registry.go:200 recursiveSearchSubset
// recursiveSearchSubset reports whether target is a subset of current.
// nil represents "all packages" — a superset of every concrete set.
// Returns true if the current set already covers everything the target needs,
// meaning no rebuild is required for recursive search purposes.
pub fn recursive_search_subset(
    target: Option<&FxHashSet<String>>,
    current: Option<&FxHashSet<String>>,
) -> bool {
    if target.is_none() {
        // Target wants all packages searched — only satisfied if current is also all.
        return current.is_none();
    }
    if current.is_none() {
        // Current searched all packages, so any concrete target is satisfied.
        return true;
    }
    set_is_subset_of(target, current)
}

// Go: ls/autoimport/registry.go:212 RegistryBucket
// PORT: see the file header. Go nil maps that are only read are empty maps
// (`paths`, `ambient_module_names`); `package_files` keeps nil (`None`)
// because Go tests it.
#[derive(Default)]
pub struct RegistryBucket {
    pub state: RefCell<BucketState>,

    // Paths maps file paths to package names. For project buckets, the package name
    // is always empty string. For node_modules buckets, this enables reverse lookup
    // from path to package for granular updates. Only paths for local workspace
    // packages (symlinked and within the workspace root) have entries here, since
    // their realpaths are outside node_modules and need reverse lookup for dirty
    // detection.
    //
    // Paths is considered immutable after the bucket is finalized.
    // It should be fully replaced rather than mutated while changing a bucket.
    pub paths: Rc<FxHashMap<tspath::Path, String>>,
    // PackageFiles maps package names to their file paths and file names.
    // All package directory names in node_modules are keys; indexed packages have
    // non-nil maps with path→fileName entries, unindexed packages have nil maps.
    // This enables efficient removal of a package's files during granular updates
    // without iterating through all entries. Only defined for node_modules buckets.
    //
    // PackageFiles is considered immutable after the bucket is finalized.
    // It should be fully replaced rather than mutated while changing a bucket.
    pub package_files: Option<Rc<FxHashMap<String, Option<FxHashMap<tspath::Path, String>>>>>,
    // ResolvedPackageNames is only defined for project buckets. It is the set of
    // package names that were resolved from imports in the project's program files.
    // This is passed to node_modules buckets so they include packages that are
    // directly imported even if not listed in package.json dependencies.
    //
    // ResolvedPackageNames is considered immutable after the bucket is finalized.
    // It should be fully replaced rather than mutated while changing a bucket.
    pub resolved_package_names: Option<Rc<FxHashSet<String>>>,
    // DependencyNames is only defined for node_modules buckets. It is the set of
    // package names that will be included in the bucket if present in the directory,
    // computed from package.json dependencies plus resolved package names from
    // active programs. If nil, all packages are included because at least one open
    // file has access to this node_modules directory without being filtered by a
    // package.json.
    //
    // DependencyNames is considered immutable after the bucket is finalized.
    // It should be fully replaced rather than mutated while changing a bucket.
    pub dependency_names: Option<Rc<FxHashSet<String>>>,
    // AmbientModuleNames is only defined for node_modules buckets. It is the set of
    // ambient module names found while extracting exports in the bucket.
    //
    // AmbientModuleNames is considered immutable after the bucket is finalized.
    // It should be fully replaced rather than mutated while changing a bucket.
    pub ambient_module_names: Rc<FxHashMap<String, Vec<String>>>,
    // Index is considered immutable after the bucket is finalized.
    // It should be cloned and replaced rather than mutated while changing a bucket.
    pub index: Option<Rc<RefCell<Index<Rc<Export>>>>>,
}

// Go: ls/autoimport/registry.go:263 newRegistryBucket
pub fn new_registry_bucket() -> Rc<RegistryBucket> {
    Rc::new(RegistryBucket {
        state: RefCell::new(BucketState {
            multiple_files_dirty: true,
            new_program_structure: NewProgramStructure::DIFFERENT_FILE_NAMES,
            ..Default::default()
        }),
        ..Default::default()
    })
}

impl RegistryBucket {
    // Go: ls/autoimport/registry.go:267 Clone
    pub fn clone_(&self) -> Rc<RegistryBucket> {
        Rc::new(RegistryBucket {
            state: RefCell::new(self.state.borrow().clone_()),
            paths: self.paths.clone(),
            package_files: self.package_files.clone(),
            resolved_package_names: self.resolved_package_names.clone(),
            dependency_names: self.dependency_names.clone(),
            ambient_module_names: self.ambient_module_names.clone(),
            index: self.index.clone(),
        })
    }

    // Go: ls/autoimport/registry.go:287 markProjectFileDirty
    // markProjectFileDirty should only be called within a Change call on the dirty map.
    // Buckets are considered immutable once in a finalized registry. Should only
    // be used for project buckets.
    pub fn mark_project_file_dirty(&self, file: &tspath::Path) {
        let mut state = self.state.borrow_mut();
        if state.has_dirty_file_besides(file) {
            state.multiple_files_dirty = true;
        } else {
            state.dirty_file = file.clone();
        }
    }

    // Go: ls/autoimport/registry.go:299 markNodeModulesDirty
    // markNodeModulesDirty should only be called within a Change call on the dirty map.
    // Buckets are considered immutable once in a finalized registry. If packageName is
    // non-empty, that package is marked for granular update. Otherwise, the entire bucket
    // is marked dirty.
    pub fn mark_node_modules_dirty(&self, package_name: &str) {
        let mut state = self.state.borrow_mut();
        if state.multiple_files_dirty {
            return;
        }
        if package_name.is_empty() {
            state.multiple_files_dirty = true;
            return;
        }
        // Track the package for granular updates
        if state.dirty_packages.is_none() {
            state.dirty_packages = Some(FxHashSet::default());
        }
        state
            .dirty_packages
            .as_mut()
            .unwrap()
            .insert(package_name.to_string());
    }
}

impl dirty::Cloneable for Rc<RegistryBucket> {
    fn clone_(&self) -> Self {
        RegistryBucket::clone_(self)
    }
}

// Go: ls/autoimport/registry.go:314 directory



// Go: ls/autoimport/registry.go:328 Registry
// PORT: Go `func(fileName string) tspath.Path` is `Rc<dyn Fn(&str) -> tspath::Path>`.
// Go `*collections.SyncMap[tspath.Path, string]` specifier caches are
// `Rc<RefCell<FxHashMap<tspath::Path, String>>>` (shared between registries).

// Go: ls/autoimport/registry.go:346 NewRegistry
// PORT: map-ls-completions 2.5 returns the value; callers share it as
// `Rc<Registry>`.


// Go: ls/autoimport/registry.go:417 BucketStats
#[derive(Clone, Debug, Default)]
pub struct BucketStats {
    pub name: String,
    pub export_count: i32,
    pub file_count: i32,
    pub state: BucketState,
    pub dependency_names: Option<Rc<FxHashSet<String>>>,
    pub package_names: Option<FxHashSet<String>>,
}

// Go: ls/autoimport/registry.go:426 CacheStats
#[derive(Clone, Debug, Default)]
pub struct CacheStats {
    pub project_buckets: Vec<BucketStats>,
    pub node_modules_buckets: Vec<BucketStats>,
    pub unique_package_count: i32,
}


// Go: ls/autoimport/registry.go:487 RegistryChange
// PORT: Go `collections.Set[lsproto.DocumentUri]` values are `FxHashSet`;
// Go `*lsutil.UserPreferences` is `Option` (nil is `None`).
#[derive(Clone, Debug, Default)]
pub struct RegistryChange {
    pub requested_file: tspath::Path,
    pub open_files: FxHashMap<tspath::Path, String>,
    pub changed: FxHashSet<lsproto::DocumentUri>,
    pub created: FxHashSet<lsproto::DocumentUri>,
    pub deleted: FxHashSet<lsproto::DocumentUri>,
    // RebuiltPrograms maps from project ID to:
    //   - true: the program was rebuilt with a different set of file names
    //   - false: the program was rebuilt but the set of file names is unchanged
    pub rebuilt_programs: FxHashMap<ProjectID, bool>,
    pub user_preferences: Option<lsutil::UserPreferences>,
}



// Go: ls/autoimport/registry.go:500 RegistryCloneHost
// PORT: Go embeds `module.ResolutionHost` and repeats its `FS()`; both come
// from the supertrait (`fs`, `get_current_directory`). Go
// `*compiler.Program` is `Rc<compiler::NewProgram>` (nil is `None`), Go
// `*packagejson.InfoCacheEntry` is `Option<Rc<..>>`, and Go `*ast.SourceFile`
// is `Node` (`Node::NIL` for nil). The project area implements it
// (`autoImportRegistryCloneHost`).
// Go nil `ProjectID` is `None`.

// Go: ls/autoimport/registry.go:510 registryBuilder
// PORT: Go `*dirty.Map` / `*dirty.MapBuilder` are the `Rc` handles that
// `dirty::new_map` / `dirty::new_map_builder` return.

// Go: ls/autoimport/registry.go:525 newRegistryBuilder
// PORT: the dirty maps copy their base maps (see `dirty::new_map`); Go shares them.


// Go: ls/autoimport/registry.go:1137 hasNewNonNodeModulesFiles
// PORT: Go `program` can be nil; it is read only for a
// `newProgramStructureDifferentFileNames` bucket, where nil panics as in Go.

// Go: ls/autoimport/registry.go:1152 isIgnoredFile
// PORT: only `FileName()` and `Path()` are read, so the program's
// `ParsedSourceFile` is passed (see the file header).

// Go: ls/autoimport/registry.go:1159 hasSymlinkToNodeModules
// hasSymlinkToNodeModules checks if a file's realpath has a symlink that points
// to a node_modules directory. This is used to skip files in the project bucket
// that would be duplicated by the node_modules bucket via their symlink.
// PORT: Go `FilesByRealpath()` and `DirectoriesByRealpath()` can be nil; the
// port's maps always exist, so those nil checks always pass.

// Go: ls/autoimport/registry.go:1210 failedAmbientModuleLookupSource
// PORT: Go `mu sync.Mutex` is dropped (one thread).
#[derive(Clone, Debug, Default)]
pub struct FailedAmbientModuleLookupSource {
    pub file_name: String,
    pub package_name: String,
}

// Go: ls/autoimport/registry.go:1216 bucketBuildResult
// PORT: Go `error` is `Option<GoError>`; Go nil sync maps and sets are
// `None`.
pub struct BucketBuildResult {
    /// Go `replaceBucket func(*RegistryBucket)`.
    pub replace_bucket: Box<dyn Fn(Rc<RegistryBucket>)>,
    pub resolution_path: tspath::Path,
    pub err: Option<GoError>,

    pub bucket: Option<Rc<RegistryBucket>>,
    // entrypoints are the resolved entrypoints from this bucket's packages,
    // to be merged into the registry-level entrypoints map.
    pub entrypoints: FxHashMap<tspath::Path, Vec<Rc<module::ResolvedEntrypoint>>>,
    // removedEntrypointPaths lists paths whose entrypoints should be removed from
    // the registry-level map before merging new entrypoints. Used for granular updates.
    pub removed_entrypoint_paths: Vec<tspath::Path>,
    // File path to filename and package name
    pub possible_failed_ambient_module_lookup_sources:
        Option<IndexMap<tspath::Path, Rc<RefCell<FailedAmbientModuleLookupSource>>>>,
    // Likely ambient module name
    pub possible_failed_ambient_module_lookup_targets: Option<IndexSet<String>>,
}

/// Go `&bucketBuildResult{replaceBucket: .., resolutionPath: ..}`.
fn new_bucket_build_result(
    replace_bucket: Box<dyn Fn(Rc<RegistryBucket>)>,
    resolution_path: tspath::Path,
) -> BucketBuildResult {
    BucketBuildResult {
        replace_bucket,
        resolution_path,
        err: None,
        bucket: None,
        entrypoints: FxHashMap::default(),
        removed_entrypoint_paths: Vec::new(),
        possible_failed_ambient_module_lookup_sources: None,
        possible_failed_ambient_module_lookup_targets: None,
    }
}


// Go: ls/autoimport/registry.go:1356 discoveredPackage
// discoveredPackage represents a package found during the discovery phase.
// It holds the resolved package.json and realpath for deduplication.
// When both a real package and a corresponding @types package exist (e.g., react + @types/react),
// both are stored so extraction can fall back to the @types package if the real package has no
// TypeScript entrypoints.

// Go: ls/autoimport/registry.go:1369 perPackageExtractionResult
// perPackageExtractionResult holds the extraction output for one physical package.
// Produced once per unique realpath during the extraction phase, then installed
// into every bucket that needs it during the bucket-building phase.
// PORT: the alias resolver callback adds to the two failed-lookup fields
// while the result is being built, so they are shared `Rc<RefCell<..>>`.
pub struct PerPackageExtractionResult {
    pub package_files: FxHashMap<tspath::Path, String>,
    pub entrypoints: Vec<Rc<module::ResolvedEntrypoint>>,
    pub exports: IndexMap<tspath::Path, Vec<Rc<Export>>>,
    pub ambient_modules: FxHashMap<String, Vec<String>>,
    pub stats_exports: i32,
    pub stats_used_checker: i32,
    pub skipped_entrypoints: i32,
    pub is_symlinked: bool,
    pub failed_ambient_module_lookup_sources:
        Rc<RefCell<IndexMap<tspath::Path, Rc<RefCell<FailedAmbientModuleLookupSource>>>>>,
    pub failed_ambient_module_lookup_targets: Rc<RefCell<IndexSet<String>>>,
}

// Go: ls/autoimport/registry.go:1383 packageExtractionResult
// packageExtractionResult holds the results of extracting exports from a set of packages.
pub struct PackageExtractionResult {
    pub exports: IndexMap<tspath::Path, Vec<Rc<Export>>>,
    pub package_files: FxHashMap<String, FxHashMap<tspath::Path, String>>,
    pub ambient_module_names: FxHashMap<String, Vec<String>>,
    pub entrypoints: Vec<Vec<Rc<module::ResolvedEntrypoint>>>,
    pub workspace_packages: FxHashSet<String>,
    pub possible_failed_ambient_module_lookup_sources:
        IndexMap<tspath::Path, Rc<RefCell<FailedAmbientModuleLookupSource>>>,
    pub possible_failed_ambient_module_lookup_targets: IndexSet<String>,
    pub stats: ExtractorStats,
    pub skipped_entrypoints_count: i32,
}


// Go: ls/autoimport/registry.go:1575 installExtractions
// installExtractions aggregates pre-extracted per-package results into a single
// packageExtractionResult for one bucket. This is the install phase of the three-phase pipeline.

