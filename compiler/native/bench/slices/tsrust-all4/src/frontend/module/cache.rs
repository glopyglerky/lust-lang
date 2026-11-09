//! Port of module/cache.go.

use crate::frontend::prelude::*;
use std::cell::Cell;
use std::sync::Arc;

// Go: module/cache.go:9 ModeAwareCache
pub type ModeAwareCache<T> = FxHashMap<ModeAwareCacheKey, T>;

// Go: module/cache.go:11 moduleResolutionCacheKey
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModuleResolutionCacheKey {
    pub containing_directory: String,
    pub module_name: String,
    pub resolution_mode: ResolutionMode,
    pub redirect_config_name: String,
}

/// The fields of a `ModuleResolutionCacheKey`: containing directory,
/// module name, resolution mode and redirect config name.
pub type ModuleKeyParts<'a> = (&'a str, &'a str, ResolutionMode, &'a str);

/// A module resolution cache key as its parts, so a lookup needs no new
/// key (`ModuleResolutionCache::get`).
// PORT: not in Go (perf). A Go key is a struct of string headers, which
// costs no allocation; a Rust key owns its strings.
pub trait ModuleKey {
    fn parts(&self) -> ModuleKeyParts<'_>;
}

impl ModuleKey for ModuleResolutionCacheKey {
    fn parts(&self) -> ModuleKeyParts<'_> {
        (
            &self.containing_directory,
            &self.module_name,
            self.resolution_mode,
            &self.redirect_config_name,
        )
    }
}

impl ModuleKey for ModuleKeyParts<'_> {
    fn parts(&self) -> ModuleKeyParts<'_> {
        *self
    }
}


// The same hash as a derived one (the fields in order), so the map order
// does not change.
impl std::hash::Hash for ModuleResolutionCacheKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.parts().hash(state);
    }
}

impl std::hash::Hash for dyn ModuleKey + '_ {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.parts().hash(state);
    }
}



impl ModuleResolutionCacheKey {
    /// The key of `parts`.
    #[must_use]
    pub fn from_parts(parts: ModuleKeyParts<'_>) -> Self {
        let (containing_directory, module_name, resolution_mode, redirect_config_name) = parts;
        ModuleResolutionCacheKey {
            containing_directory: containing_directory.to_string(),
            module_name: module_name.to_string(),
            resolution_mode,
            redirect_config_name: redirect_config_name.to_string(),
        }
    }
}

// Go: module/cache.go:18 moduleResolutionCache
// PORT: Go `collections.SyncMap` is a plain map behind a `RefCell`, so the
// `DefaultResolver` methods can take `&self`. Cached Go pointers are `Arc`: parse
// workers share them (`SharedResolutionCache`), and checker threads read
// the program's resolutions with no copy (`GoSharedState`).


// Go: module/cache.go:30 typeRefDirectiveResolutionCacheKey
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct TypeRefDirectiveResolutionCacheKey {
    pub containing_directory: String,
    pub type_reference_name: String,
    pub resolution_mode: ResolutionMode,
    pub redirect_config_name: String,
    pub from_inferred_types_containing_file: bool,
}

// Go: module/cache.go:38 typeRefDirectiveResolutionCache
// PORT: see `ModuleResolutionCache`.


// Go: module/cache.go:50 parsedPatternsCache
// PORT: Go keys the `SyncMap` by the `*OrderedMap` of `paths`, and the key
// keeps that map alive. The Rust `paths` map is a field of the
// `CompilerOptions`, so the key is the address of that field (0 for a nil
// map), and the entry holds the options `Rc` that owns it.


// Go: module/cache.go:62 caches
// PORT: Go `*packagejson.InfoCache` is shared between resolvers
// (`ResolverOptions.PackageJsonCache`), so it is `Rc<InfoCache>`. `InfoCache`
// has interior mutability, like the Go `SyncMap`.


/// The keys of the previous load, which the resolve-ahead workers resolve
/// in order, and who resolves each one: the loader takes a key that no
/// worker has started, so no key is resolved twice, and it waits for a key
/// that a worker resolves now.
// PORT: not in Go (perf).
pub struct AheadQueue {
    pub keys: Arc<KeyList>,
    /// The index of the next key for a worker.
    pub next: std::sync::atomic::AtomicUsize,
    /// Per key: `KEY_FREE`, `KEY_WORKER`, `KEY_DONE` or `KEY_LOADER`.
    states: Box<[std::sync::atomic::AtomicU8]>,
    /// The key that the loader waits for on `done_cv`, or `usize::MAX`.
    waiting: std::sync::atomic::AtomicUsize,
    done_lock: std::sync::Mutex<()>,
    done_cv: std::sync::Condvar,
    /// A worker panicked outside a resolution (`fail`).
    failed: std::sync::atomic::AtomicBool,
}

const KEY_FREE: u8 = 0;
/// A worker resolves the key now.
const KEY_WORKER: u8 = 1;
/// A worker ended the key: its answer is published, unless the
/// resolution could not be shared or panicked.
const KEY_DONE: u8 = 2;
/// The loader resolves the key itself.
const KEY_LOADER: u8 = 3;

/// How many times the loader checks a key that a worker resolves before
/// it sleeps (`AheadQueue::wait_or_take`): about 2 to 4 microseconds.
///
/// A longer spin does not make the waits shorter. On import edits (effect
/// and query-core, two hosts, 200 loads each) the loader waited 14 to 28
/// times per load, 30 to 40 microseconds each, 0.4 to 1.0 ms per load in
/// all. A spin of up to 50 or 200 microseconds took the sleeps away but
/// not the wait time: the loader has caught up with the workers there, so
/// it waits for their work, not for the wake (loadcuts1, cut 4).
const WAIT_SPINS: u32 = 64;

/// How many keys after the cursor `AheadQueue::find` looks at. The loader
/// meets the keys of the previous load in its order, less removed keys
/// and with new ones between them.
const FIND_AHEAD: usize = 4;

impl AheadQueue {

    /// A worker panicked outside a resolution (compiler/resolve_ahead.rs
    /// `run_task`): the loader takes no more answers of this load and
    /// resolves the rest of its keys itself, and the workers take no more
    /// keys. It does not close the job: the workers still store their
    /// lookups until the load ends.
    pub fn fail(&self) {
        self.failed
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// `fail` was called.
    #[must_use]
    pub fn failed(&self) -> bool {
        self.failed.load(std::sync::atomic::Ordering::Relaxed)
    }


    /// A worker takes the next key that the loader did not take: its index
    /// and parts. `None` when no key is left.
    pub fn take_next(&self) -> Option<(usize, (&str, &str, ResolutionMode))> {
        use std::sync::atomic::Ordering;
        loop {
            let index = self.next.fetch_add(1, Ordering::Relaxed);
            let key = self.keys.get(index)?;
            if self.states[index]
                .compare_exchange(KEY_FREE, KEY_WORKER, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return Some((index, key));
            }
        }
    }


    /// The index of the loader's key `key`, from `cursor` on, and moves
    /// the cursor after it.
    fn find(&self, cursor: &Cell<usize>, key: (&str, &str, ResolutionMode)) -> Option<usize> {
        let start = cursor.get();
        let index = (start..self.keys.len().min(start + FIND_AHEAD))
            .find(|&index| self.keys.get(index) == Some(key))?;
        cursor.set(index + 1);
        Some(index)
    }

}




/// The parts of a package scope (Go `GetPackageScopeForPath`) that Go
/// `loadSourceFileMetaData` reads: the directory of the package.json and
/// its `type` field. `None` (no scope) when no package.json exists in the
/// directory or above it.
// PORT: not in Go (resolve ahead). A worker sends these parts to the loader,
// as its package.json parses stay on the worker.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageScope {
    /// Go `GetDirectory()` of the scope.
    pub package_directory: String,
    /// The `type` field, when it is there and valid.
    pub type_: Option<String>,
}

impl PackageScope {
}

/// The module resolution keys of one program load that have no redirect,
/// and the directories whose package scope it found, in the load's order
/// (`AheadLink::keys`). A package scope key has an empty module name (a
/// module key never has one) and no mode (`push_scope`). The keys share
/// one text, so the loader records a key with no allocation of its own,
/// and the list frees in two.
// PORT: not in Go (perf).
#[derive(Default)]
pub struct KeyList {
    text: String,
    /// Per key: the end of its containing directory and of its module name
    /// in `text`, and its resolution mode. A key starts where the one
    /// before it ends.
    ends: Vec<(usize, usize, ResolutionMode)>,
    /// How many of the keys are package scope keys.
    scopes: usize,
}

impl KeyList {
    /// An empty list with room for the keys of `like`.
    #[must_use]
    pub fn with_capacity_of(like: &KeyList) -> Self {
        KeyList {
            text: String::with_capacity(like.text.len()),
            ends: Vec::with_capacity(like.ends.len()),
            scopes: 0,
        }
    }

    pub fn push(&mut self, containing_directory: &str, module_name: &str, mode: ResolutionMode) {
        self.text.push_str(containing_directory);
        let directory_end = self.text.len();
        self.text.push_str(module_name);
        self.ends.push((directory_end, self.text.len(), mode));
    }

    /// Records the package scope key of `directory`.
    pub fn push_scope(&mut self, directory: &str) {
        self.push(directory, "", RESOLUTION_MODE_NONE);
        self.scopes += 1;
    }

    /// The number of keys, package scope keys included.
    #[must_use]
    pub fn len(&self) -> usize {
        self.ends.len()
    }

    /// The number of package scope keys.
    #[must_use]
    pub fn scopes(&self) -> usize {
        self.scopes
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ends.is_empty()
    }

    /// The containing directory, module name and mode of key `index`.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<(&str, &str, ResolutionMode)> {
        let &(directory_end, end, mode) = self.ends.get(index)?;
        let start = index.checked_sub(1).map_or(0, |before| self.ends[before].1);
        Some((
            &self.text[start..directory_end],
            &self.text[directory_end..end],
            mode,
        ))
    }
}

/// What the loader did with the keys of a resolve-ahead load that its own
/// cache did not have.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AheadStats {
    /// Worker answers that passed the check.
    pub taken: usize,
    /// Worker answers that failed the check.
    pub rejected: usize,
    /// Keys with no worker answer (not resolved yet, new or not shareable),
    /// or after a worker panic (`AheadQueue::fail`).
    pub missing: usize,
    /// Keys whose worker answer the loader waited for.
    pub waited: usize,
    /// Of `waited`: the waits that slept after the spin.
    pub slept: usize,
    /// The time of all the waits, in nanoseconds.
    pub wait_ns: u64,
    /// `taken`, `rejected` and `missing` of the package scope keys (the
    /// other counts have both kinds of keys).
    pub scopes_taken: usize,
    pub scopes_rejected: usize,
    pub scopes_missing: usize,
}


// Go: module/cache.go:18 moduleResolutionCache and :40
// typeRefDirectiveResolutionCache, the `SyncMap`s themselves.
// PORT: Go shares one resolver, and so these maps, between all parse tasks
// of a program. The Rust loader resolves on one thread with `Rc` values
// (`Caches`), and each parse worker has its own resolver. This is the part
// they share: the loader's resolver reads what the workers resolved ahead
// of it. A resolution is a function of its key (the key has the redirect
// config) and of the file system, so any resolver stores the same answer
// that the loader would find; first answer wins, as in Go. Only a program
// that resolves on the plain OS file system, with no project references
// and no traced resolution, shares one (`process_all_program_files`).
//
// Go also shares one package.json cache, which the build info lists
// (`Program::package_json_cache_entries`). So each answer carries the
// package.json lookups that made it (`SharedResolution`), and the loader
// adds them to its own package.json cache.

/// A parse worker's answer in the `SharedResolutionCache`.
#[derive(Clone)]
pub struct SharedResolution<T> {
    pub value: T,
    /// The package.json lookups of the resolution (`package_json_log`).
    pub package_jsons: Arc<[PackageJsonLookup]>,
    /// The file system lookups of the resolution that the `tsc -b` host's
    /// cache did not have (`note_worker_lookup`). `None` when the worker
    /// does not log them (no `tsc -b` host).
    pub lookups: Option<Arc<[StatLookup]>>,
    /// The file system calls of a resolve-ahead worker's resolution, with
    /// their answers (`AheadCall`). `None` from other parse workers. The
    /// loader takes a resolve-ahead answer only with them.
    pub ahead: Option<Arc<[AheadCall]>>,
}

/// One file system call of a resolve-ahead worker's resolution
/// (compiler/resolve_ahead.rs), with its answer. Before the loader takes
/// the answer, it checks each call on its own file system and replays the
/// side effects of the call (`AheadLink::accept`). The name of a
/// `FileExists` or `DirectoryExists` call is its path (the worker does not
/// share an answer with another name), so the loader's caches by name and
/// by path both find it.
// PORT: not in Go (perf). Go resolves in each parse task on the host's
// file system, which tracks the calls itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AheadCall {
    /// `file_exists` of `path`. `known`: the worker took the answer from
    /// what the workers found in earlier loads, not from the OS during
    /// this load, so the loader asks its own file system.
    FileExists {
        path: Path,
        exists: bool,
        known: bool,
    },
    /// `directory_exists` of `path`.
    DirectoryExists { path: Path, exists: bool },
    /// `realpath` of `name`.
    Realpath { name: String, real: String },
    /// `read_file`: the xxh3 hash of the text, `None` when the file could
    /// not be read.
    Read {
        file_name: String,
        hash: Option<u128>,
    },
    /// The calls of Go `getPackageJsonInfo` for one package.json cache
    /// entry (no `PackageJson` in them). One worker's answers that read the
    /// entry share them, so the loader checks them once per load.
    PackageJson(Arc<[AheadCall]>),
    /// A `DirectoryExists` or `Realpath` that all the answers of the job
    /// share (compiler/resolve_ahead.rs `WorkerStats`), so the loader checks
    /// it once per load.
    Shared(Arc<AheadCall>),
}

impl AheadCall {
    /// Calls `f` with each call of `calls`, the calls of a `PackageJson`
    /// or `Shared` in its place.
    pub fn each(calls: &[AheadCall], f: &mut impl FnMut(&AheadCall)) {
        for call in calls {
            match call {
                AheadCall::PackageJson(group) => AheadCall::each(group, f),
                AheadCall::Shared(call) => AheadCall::each(std::slice::from_ref(&**call), f),
                call => f(call),
            }
        }
    }
}

/// How the resolve-ahead call log of a resolution ended
/// (`Caches::take_ahead_log`).
pub enum AheadLogEnd {
    /// This thread does not log (it is no resolve-ahead worker).
    NotLogged,
    /// The resolution made a call that the loader cannot check (a read of
    /// an open file, a directory listing, a stat). Its answer is not
    /// published.
    Unshareable,
    Logged(Arc<[AheadCall]>),
}

/// The state of a resolve-ahead worker thread.
struct AheadThread {
    current_directory: String,
    use_case_sensitive_file_names: bool,
    /// The calls of the resolution that runs now. `None` between
    /// resolutions.
    calls: Option<Vec<AheadCall>>,
    /// False when the resolution made a call that the loader cannot check.
    shareable: bool,
    /// The hash of each file that this worker read, by name. The
    /// package.json cache of the worker's resolver keeps the parses of
    /// these texts (from job to job, compiler/resolve_ahead.rs), and a
    /// later resolution that reads the cache logs the read again
    /// (`Caches::log_package_json`).
    reads: FxHashMap<String, Option<u128>>,
    /// The calls of each package.json cache entry of this job, by the
    /// entry's address (the cache keeps each entry for the whole job).
    /// `None`: the entry's file was not read from a file that the loader
    /// can check.
    package_jsons: FxHashMap<usize, Option<Arc<[AheadCall]>>>,
    /// The package.json files whose cache entries the worker kept from an
    /// earlier job (`begin_ahead_thread`).
    kept: FxHashSet<String>,
    /// Whether a directory exists now on the worker's file system, with
    /// no log (`begin_ahead_thread`).
    directory_exists: Rc<dyn Fn(&str) -> bool>,
}

thread_local! {
    /// Set on a resolve-ahead worker thread (`begin_ahead_thread`).
    static AHEAD: RefCell<Option<AheadThread>> = const { RefCell::new(None) };
}

/// Makes this thread a resolve-ahead worker: its resolutions log their file
/// system calls (`note_ahead_call`). `current_directory` and
/// `use_case_sensitive_file_names` make the paths, as the host's `to_path`.
/// `reads` has the hash of the text of each package.json that the
/// worker's package.json cache has already; `kept` names those that the
/// cache kept from an earlier job. `directory_exists` answers whether a
/// directory exists now, as the worker's file system does, with no log.
pub fn begin_ahead_thread(
    current_directory: &str,
    use_case_sensitive_file_names: bool,
    reads: FxHashMap<String, Option<u128>>,
    kept: FxHashSet<String>,
    directory_exists: Rc<dyn Fn(&str) -> bool>,
) {
    AHEAD.with(|ahead| {
        *ahead.borrow_mut() = Some(AheadThread {
            current_directory: current_directory.to_string(),
            use_case_sensitive_file_names,
            calls: None,
            shareable: true,
            reads,
            package_jsons: FxHashMap::default(),
            kept,
            directory_exists,
        });
    });
}

/// True on a resolve-ahead worker (`begin_ahead_thread`).
#[must_use]
pub fn on_ahead_thread() -> bool {
    AHEAD.with(|ahead| ahead.borrow().is_some())
}

/// Ends `begin_ahead_thread`: the hashes of the files that the worker read
/// (`AheadThread::reads`).
#[must_use]
pub fn end_ahead_thread() -> FxHashMap<String, Option<u128>> {
    AHEAD
        .with(|ahead| ahead.borrow_mut().take())
        .map(|state| state.reads)
        .unwrap_or_default()
}

/// Logs `call` in the resolution that runs on this thread, if it logs.
pub fn note_ahead_call(call: AheadCall) {
    AHEAD.with(|ahead| {
        if let Some(calls) = ahead
            .borrow_mut()
            .as_mut()
            .and_then(|state| state.calls.as_mut())
        {
            calls.push(call);
        }
    });
}

/// Logs a read of `file_name` (`hash` of its text, `None` when it could not
/// be read) in the resolution that runs on this thread, and keeps the hash
/// for later reads of the same parse.
pub fn note_ahead_read(file_name: &str, hash: Option<u128>) {
    AHEAD.with(|ahead| {
        if let Some(state) = ahead.borrow_mut().as_mut() {
            state.reads.insert(file_name.to_string(), hash);
            if let Some(calls) = state.calls.as_mut() {
                calls.push(AheadCall::Read {
                    file_name: file_name.to_string(),
                    hash,
                });
            }
        }
    });
}



/// Marks the resolution that runs on this thread as not shareable: it made
/// a call that the loader cannot check. With `file_name`, it read that
/// file from a source that the loader cannot check (an open file), so a
/// later resolution that uses its parse is not shareable either.
pub fn note_ahead_unshareable(file_name: Option<&str>) {
    AHEAD.with(|ahead| {
        if let Some(state) = ahead.borrow_mut().as_mut() {
            state.shareable = false;
            if let Some(file_name) = file_name {
                state.reads.remove(file_name);
            }
        }
    });
}

/// A lookup that Go `cachedvfs` caches (all but `Stat`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatKind {
    FileExists,
    DirectoryExists,
    Realpath,
    Entries,
}

/// One cached lookup of a parse worker's resolution, without its value.
/// The value is in the worker cache of the program load
/// (`BuildStatCache`).
#[derive(Clone, Debug)]
pub struct StatLookup {
    pub kind: StatKind,
    pub path: String,
}

thread_local! {
    /// True on a parse worker thread whose file system reads the `tsc -b`
    /// host's cache (`set_worker_lookup_log`).
    static WORKER_LOOKUP_LOG: Cell<bool> = const { Cell::new(false) };
    /// The lookups of the parse worker resolution that runs on this thread
    /// now (`Caches::start_package_json_log` to `take_package_json_log`).
    static WORKER_LOOKUPS: RefCell<Option<Vec<StatLookup>>> = const { RefCell::new(None) };
}

/// Makes the resolutions of this parse worker thread log their file system
/// lookups (`note_worker_lookup`) or not.
// PORT: not in Go. Go parse tasks share the host's cachedvfs, so the
// lookups of each resolution are in it. A `tsc -b` parse worker keeps its
// lookups out of the host's cache, and the loader adds the lookups of the
// answers it takes (`BuildStatCache`).
pub fn set_worker_lookup_log(on: bool) {
    WORKER_LOOKUP_LOG.with(|log| log.set(on));
}

/// Records a lookup of the parse worker resolution that runs on this
/// thread, if one runs and this thread logs lookups.
pub fn note_worker_lookup(kind: StatKind, path: &str) {
    WORKER_LOOKUPS.with(|log| {
        if let Some(log) = log.borrow_mut().as_mut() {
            log.push(StatLookup {
                kind,
                path: path.to_string(),
            });
        }
    });
}

/// One Go `getPackageJsonInfo` call of a parse worker's resolution: the
/// package.json cache entry that it read or stored. The parsed contents stay
/// on the worker; a lookup of the metadata scope walk carries the text of
/// the first read (`read`).
#[derive(Clone, Debug)]
pub struct PackageJsonLookup {
    pub package_directory: String,
    pub directory_exists: bool,
    /// The package.json file exists (`InfoCacheEntry::exists`).
    pub exists: bool,
    /// The first read of the load for the entry's package.json, which the
    /// loader keeps in its own cache when it takes the lookup
    /// (`Caches::adopt_worker_package_jsons`). Only the lookups of a parse
    /// worker's package scope walk for the metadata of a file have it
    /// (`WorkerPackageJsonReads::attach`); the loader keeps the other reads
    /// at the end of the load (`SharedResolutionCache::end_package_json_reads`).
    pub read: Option<Arc<WorkerPackageJsonRead>>,
}

/// A parse worker's reads of its package.json cache entries.
// PORT: not in Go (see `WorkerPackageJsonRead`).
#[derive(Default)]
pub struct WorkerPackageJsonReads {
    /// The read of the load for each entry
    /// (`SharedResolutionCache::store_package_json_read`), by the address of
    /// the entry. The worker's cache keeps each entry while the resolver
    /// lives, so an address names one entry.
    reads: RefCell<FxHashMap<usize, Arc<WorkerPackageJsonRead>>>,
    /// True while the worker finds the package scope of a file's directory
    /// for its metadata (Go `loadSourceFileMetaData`, which the loader runs
    /// in Go): the lookups then carry their reads, for the loader's cache.
    pub attach: Cell<bool>,
}

/// The first read of one package.json by the parse workers of a load (Go
/// `getPackageJsonInfo`, module/resolver.go:1755): the parts of the
/// package.json cache entry that it made, with the text of the file. The
/// lookups of the file that carry a read share it. A worker's entry has `Rc`
/// contents, which stay on the worker's thread, so the loader takes the
/// text (`adopt`), and its cache parses the text when a lookup first asks
/// for the entry (`InfoCache::get`).
// PORT: not in Go. Go's parse tasks share the resolver's one package.json
// cache, which keeps the first entry of each file (packagejson/cache.go:190
// `Set`).
pub struct WorkerPackageJsonRead {
    package_directory: String,
    directory_exists: bool,
    /// Set when the loader has taken the read.
    taken: std::sync::atomic::AtomicBool,
    /// The text of the package.json, until the loader takes it. `None`:
    /// the package.json does not exist.
    text: std::sync::Mutex<Option<Box<str>>>,
}

impl std::fmt::Debug for WorkerPackageJsonRead {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("WorkerPackageJsonRead")
    }
}

impl WorkerPackageJsonRead {

}

thread_local! {
    /// The last package.json that the resolver of the parse worker on this
    /// thread read (`note_worker_package_json_read`): its file name and
    /// text. `log_package_json` takes it for the entry that the read makes.
    static WORKER_PACKAGE_JSON_READ: RefCell<Option<(String, Box<str>)>> =
        const { RefCell::new(None) };
}

#[cfg(test)]
thread_local! {
    /// The reads that loaders on this thread took from parse workers
    /// (`WorkerPackageJsonRead::adopt`), for tests.
    static ADOPTED_PACKAGE_JSONS: Cell<usize> = const { Cell::new(0) };
}

/// The reads that loaders on this thread took from parse workers so far
/// (`WorkerPackageJsonRead::adopt`), for tests.
#[cfg(test)]
pub(crate) fn adopted_package_jsons() -> usize {
    ADOPTED_PACKAGE_JSONS.with(Cell::get)
}

#[cfg(test)]
thread_local! {
    /// A test's choice to make the loads on this thread wait for the
    /// answers of the parse workers (`set_answer_wait`).
    static ANSWER_WAIT: Cell<bool> = const { Cell::new(false) };
}

/// Makes the loader's resolver on this thread wait in
/// `SharedResolutionCache::get_module` until a parse worker stores the
/// answer of the key (up to 60 s), so the loader takes the worker's answer
/// whatever the timing (tests). Only for keys that a worker resolves: a
/// key that none resolves waits the 60 s, then the loader resolves it.
#[cfg(test)]
pub(crate) fn set_answer_wait(on: bool) {
    ANSWER_WAIT.with(|wait| wait.set(on));
}


/// One entry of `DefaultResolver::package_json_cache_entries`: the parts
/// of a package.json cache entry (Go `*packagejson.InfoCacheEntry`) that the
/// build info reads.
// PORT: Go yields the cache entries. The entries of the parse workers'
// lookups (`Caches::worker_package_jsons`) have no contents here, so the
// callback gets these parts of each entry instead.
#[derive(Clone, Copy, Debug)]
pub struct PackageJsonCacheEntry<'e> {
    /// Go `GetDirectory()`.
    pub package_directory: &'e str,
    pub directory_exists: bool,
    /// Go `Exists()`: the package.json file was read.
    pub exists: bool,
}





// Go: module/cache.go:74 newCaches

// Go: module/cache.go:84 getRedirectConfigName

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::vfs::osvfs_fs;

    struct TestHost {
        fs: Rc<dyn Fs>,
        current_directory: String,
    }

    impl ResolutionHost for TestHost {
        fn fs(&self) -> &dyn Fs {
            &*self.fs
        }

        fn get_current_directory(&self) -> &str {
            &self.current_directory
        }
    }

    /// A bundler resolver on the OS file system of this thread, linked to
    /// `shared` when given.
    fn test_resolver(dir: &str, shared: Option<SharedResolutionLink>) -> DefaultResolver {
        let host: Rc<dyn ResolutionHost> = Rc::new(TestHost {
            fs: osvfs_fs(),
            current_directory: dir.to_string(),
        });
        let options = Rc::new(CompilerOptions {
            module: ModuleKind::ES_NEXT,
            module_resolution: ModuleResolutionKind::BUNDLER,
            ..Default::default()
        });
        let mut resolver = new_resolver(ResolverOptions {
            host: Some(host),
            compiler_options: Some(options),
            ..Default::default()
        });
        resolver.caches.shared = shared;
        resolver
    }

    /// The imports of `src/deep/index.ts`. `@types/node` is a type
    /// reference directive.
    const NAMES: &[&str] = &[
        "pkg-a",
        "pkg-a/sub",
        "@scope/pkg-b",
        "missing-pkg",
        "@scope/missing",
        "./local",
        "@types/node",
    ];

    fn resolve(resolver: &DefaultResolver, dir: &str, names: &[&str]) {
        let containing_file = format!("{dir}/src/deep/index.ts");
        for name in names {
            if let Some(type_reference) = name.strip_prefix("@types/") {
                let _ = resolver.resolve_type_reference_directive(
                    type_reference,
                    &containing_file,
                    ModuleKind::ES_NEXT,
                    None,
                );
            } else {
                let _ =
                    resolver.resolve_module_name(name, &containing_file, ModuleKind::ES_NEXT, None);
            }
        }
    }

    /// The package.json cache entries that the build info reads, sorted.
    fn package_json_entries(resolver: &DefaultResolver) -> Vec<(String, String, bool, bool)> {
        let mut entries = Vec::new();
        resolver.package_json_cache_entries(|key, entry| {
            entries.push((
                key.0.clone(),
                entry.package_directory.to_string(),
                entry.directory_exists,
                entry.exists,
            ));
            true
        });
        entries.sort();
        entries
    }

    /// The loader's resolver takes every answer from parse workers on
    /// other threads, and its package.json cache then holds the same
    /// entries as a resolver that resolved all names itself (the one Go
    /// cache of all parse tasks).
    #[test]
    fn loader_takes_worker_package_json_lookups() {
        let root = std::env::temp_dir().join(format!(
            "ts_goport_shared_package_jsons_{}",
            std::process::id()
        ));
        let files = [
            ("src/deep/local.ts", "export {};"),
            (
                "node_modules/pkg-a/package.json",
                r#"{ "name": "pkg-a", "version": "1.0.0", "types": "index.d.ts" }"#,
            ),
            ("node_modules/pkg-a/index.d.ts", "export {};"),
            ("node_modules/pkg-a/sub/index.d.ts", "export {};"),
            (
                "node_modules/@scope/pkg-b/package.json",
                r#"{ "name": "@scope/pkg-b", "version": "2.0.0", "types": "lib/b.d.ts" }"#,
            ),
            ("node_modules/@scope/pkg-b/lib/b.d.ts", "export {};"),
            (
                "node_modules/@types/node/package.json",
                r#"{ "name": "@types/node", "version": "20.0.0" }"#,
            ),
            ("node_modules/@types/node/index.d.ts", "export {};"),
        ];
        for (path, text) in files {
            let path = root.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        let dir = root.to_string_lossy().replace('\\', "/");

        let alone = test_resolver(&dir, None);
        resolve(&alone, &dir, NAMES);
        let expected = package_json_entries(&alone);
        assert!(expected.iter().any(|entry| entry.3), "{expected:?}");
        assert!(
            expected
                .iter()
                .any(|entry| !entry.3 && entry.0.contains("/node_modules/")),
            "{expected:?}"
        );

        let shared = Arc::new(SharedResolutionCache::default());
        std::thread::scope(|scope| {
            for names in NAMES.chunks(3) {
                let link = SharedResolutionLink {
                    cache: shared.clone(),
                    publish: true,
                };
                let dir = dir.as_str();
                scope.spawn(move || resolve(&test_resolver(dir, Some(link)), dir, names));
            }
        });
        let loader = test_resolver(
            &dir,
            Some(SharedResolutionLink {
                cache: shared,
                publish: false,
            }),
        );
        resolve(&loader, &dir, NAMES);
        // Every answer came from a worker, so the loader made no lookup.
        let mut own = 0;
        loader.caches.package_json_info_cache.range(|_, _| {
            own += 1;
            true
        });
        assert_eq!(own, 0);
        let entries = package_json_entries(&loader);
        std::fs::remove_dir_all(&root).unwrap();
        assert_eq!(entries, expected);
    }

    /// Go `InfoCache.Set` keeps the first entry of each package.json
    /// (packagejson/cache.go:190), and Go's parse tasks and loader share
    /// that one cache. The parse workers' shared cache keeps the first read
    /// of each file (`store_package_json_read`). The loader's cache keeps the
    /// entry that it has when it takes worker reads
    /// (`adopt_worker_package_jsons`): its own earlier read, or the first
    /// worker read that it took, in lookup order.
    #[test]
    fn package_json_reads_keep_the_first() {
        let root = std::env::temp_dir().join(format!(
            "ts_goport_package_json_first_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(root.join("b")).unwrap();
        std::fs::write(root.join("b/package.json"), r#"{ "name": "b-loader" }"#).unwrap();
        let dir = root.to_string_lossy().replace('\\', "/");
        let read = |package: &str, name: &str| WorkerPackageJsonRead {
            package_directory: format!("{dir}/{package}"),
            directory_exists: true,
            taken: std::sync::atomic::AtomicBool::new(false),
            text: std::sync::Mutex::new(Some(format!(r#"{{ "name": "{name}" }}"#).into())),
        };
        let lookup = |read: Arc<WorkerPackageJsonRead>| PackageJsonLookup {
            package_directory: read.package_directory.clone(),
            directory_exists: true,
            exists: true,
            read: Some(read),
        };

        let shared = SharedResolutionCache::default();
        let a = format!("{dir}/a/package.json");
        let first = shared.store_package_json_read(&a, || read("a", "a-first"));
        let second = shared.store_package_json_read(&a, || read("a", "a-second"));
        assert!(
            Arc::ptr_eq(&first, &second),
            "the workers keep the first read"
        );

        let loader = test_resolver(&dir, None);
        let name = |package: &str| {
            loader
                .get_package_scope_for_path(&format!("{dir}/{package}"))
                .and_then(|entry| entry.contents.clone())
                .map(|contents| contents.fields.header_fields.name.get_value().0)
        };
        // The loader's own read of b comes before the adoption.
        assert_eq!(name("b").as_deref(), Some("b-loader"));
        std::fs::remove_dir_all(&root).unwrap();
        loader.caches.adopt_worker_package_jsons(&[
            lookup(first),
            lookup(second),
            lookup(Arc::new(read("b", "b-worker"))),
            lookup(Arc::new(read("c", "c-first"))),
            lookup(Arc::new(read("c", "c-second"))),
        ]);
        assert_eq!(
            [name("a"), name("b"), name("c")],
            [
                Some("a-first".to_string()),
                Some("b-loader".to_string()),
                Some("c-first".to_string())
            ]
        );
    }

    /// The loader's wait for a key in a worker's hands (`wait_or_take`): a
    /// free key is the loader's, an ended key needs no sleep, and a key
    /// that a worker holds past the spin makes the loader sleep until the
    /// worker's `done`. The load stats count the waits, the sleeps and the
    /// time.
    #[test]
    fn loader_waits_spin_then_sleep() {
        let mut keys = KeyList::default();
        for name in ["./a", "./b", "./c"] {
            keys.push("/p/", name, ResolutionMode::NONE);
        }
        let queue = Arc::new(AheadQueue::new(Arc::new(keys)));
        let mut stats = AheadStats::default();

        // Key 0 is free: the loader takes it, and no worker starts it.
        assert!(!queue.wait_or_take(0, &mut stats));
        assert_eq!(stats, AheadStats::default());
        assert_eq!(queue.take_next().map(|(index, _)| index), Some(1));

        // Key 1 ended: the answer is there, with no sleep.
        queue.done(1);
        assert!(queue.wait_or_take(1, &mut stats));
        assert_eq!((stats.waited, stats.slept), (1, 0));

        // Key 2 stays with a worker until the loader sleeps: the loader
        // spins, then sleeps until the worker ends it. The worker waits for
        // the loader's `waiting` mark, which the loader sets only after the
        // spin, so the order does not depend on the host's load. The 5 ms
        // after the mark is the least time that the wait can count. If the
        // mark does not come in 60 s (the loader did not sleep), the worker
        // still ends the key, so the loader cannot hang, and the test fails.
        assert_eq!(queue.take_next().map(|(index, _)| index), Some(2));
        let worker = {
            let queue = queue.clone();
            std::thread::spawn(move || {
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
                let mut marked = true;
                while queue.waiting.load(std::sync::atomic::Ordering::SeqCst) != 2 {
                    if std::time::Instant::now() > deadline {
                        marked = false;
                        break;
                    }
                    std::thread::yield_now();
                }
                if marked {
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                queue.done(2);
                marked
            })
        };
        assert!(queue.wait_or_take(2, &mut stats));
        assert!(
            worker.join().unwrap(),
            "the loader set no waiting mark in 60 s"
        );
        assert_eq!((stats.waited, stats.slept), (2, 1));
        assert!(stats.wait_ns >= 5_000_000, "{stats:?}");
    }
}
