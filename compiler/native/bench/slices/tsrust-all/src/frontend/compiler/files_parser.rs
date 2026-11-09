//! Go: compiler/filesparser.go (parse tasks, the files parser and
//! `getProcessedFiles`).

use super::file_loader::SyntheticImports;
use crate::frontend::prelude::*;
use crate::gostd::slices::stable_sort_by;
use crate::program::ThreadBudget;
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering as AtomicOrdering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};

use crate::contentmapper::{};

// Go: filesparser.go:19 parseTask
// PORT: Go `*parseTask` is shared by the root task list, sub task lists,
// `parseTaskData.tasks` and `loadedTask`, and changed through all of them.
// It is `Rc<RefCell<ParseTask>>` (`ParseTaskRef`).



// Go: filesparser.go:208 resolvedRef


// Go: filesparser.go:229 filesParser
// PORT: Go `core.WorkGroup` is single threaded here (contract 10). Go
// `singleThreadedWorkGroup` keeps queued functions in a slice and
// `RunAndWait` pops the last one first, so `queue` is a stack with the same
// order. The Go `sync.Pool` of `parseTaskData` values is not ported: a new
// value is made only when the path is new, which is the same result.
// PORT: the parses run in parallel, like the Go work group: parse workers
// parse queued files ahead of the loader (`run_prefetch_worker`), and the
// loader takes their results (`take_prefetched`). The loader still
// loads files in the serial order, so store ids, resolution order and file
// order do not change.

// PORT: Go's garbage collector frees the parse tasks with the loader. Here
// a loaded task holds its sub tasks, and a sub task of a file that was
// queued before holds the task that loaded the file (`loaded_task`). When
// files import each other, these links make an `Rc` cycle that is never
// freed. Every task that has links is in `task_data_by_path` (only those
// tasks load), so taking their links out lets the whole graph go
// (`ParseTaskLinks`). On the dispatch thread of the LSP server that free
// waits until the answer is sent (`gostd::local::drop_later`). A
// one-program process does not drop the parser
// (`with_loader_state_forgotten`).



// Go: filesparser.go:243 getParseTaskData
// PORT: Go takes the value from `parseTaskDataPool`; `putParseTaskData`
// (filesparser.go:226) returns an unused one. No pool is needed here.

// Go: filesparser.go:255 parseTaskData
// PORT: Go iterates `tasks` (a Go map) in random order. `IndexMap` keeps
// insertion order. The map holds more than one task only when one path is
// reached through file names that differ in casing.






// ──────────────────────────────────────────────────────────────────────
// Parse prefetch
// ──────────────────────────────────────────────────────────────────────
//
// PORT: Go runs `parseTask.load` (read, parse, resolve, queue the
// subtasks) in one goroutine per task. The Rust loader keeps the Go
// single-threaded order, because store ids follow the load order. Parse
// workers do the rest of Go's parallel work ahead of it: they read and
// parse queued files, resolve their imports with their own resolvers, and
// queue the files they find. The loader checks every worker result
// (`take_prefetched`) and reads the answers the workers resolved
// (`SharedResolutionCache`), so the output is the same.






/// How many of the largest root files per parse worker the workers parse
/// before the other jobs (`PrefetchShared::rank_roots`).
const ROOT_RANK_PER_WORKER: usize = 2;











thread_local! {
    /// A test's choice of `load_prep_enabled` for its loads on this thread.
    static LOAD_PREP: Cell<Option<bool>> = const { Cell::new(None) };
    /// The preps that loads on this thread took (`FilePrep`), for tests.
    static PREPS_TAKEN: Cell<usize> = const { Cell::new(0) };
}

/// True when the parse workers of a load on this thread make preps for
/// the loader (`FilePrep`): unless `GOPORT_LOAD_PREP` is `0` (an A/B
/// switch), or a test chose (`set_load_prep`).
fn load_prep_enabled() -> bool {
    LOAD_PREP
        .with(Cell::get)
        .unwrap_or_else(|| std::env::var_os("GOPORT_LOAD_PREP").is_none_or(|value| value != "0"))
}

/// Sets `load_prep_enabled` for the loads on this thread (tests).
#[cfg(test)]
pub(crate) fn set_load_prep(on: Option<bool>) {
    LOAD_PREP.with(|prep| prep.set(on));
}

#[cfg(test)]
thread_local! {
    /// A test's choice to make the loads on this thread wait in
    /// `take_prefetched_meta` for a job that is still queued.
    static META_WAIT: Cell<bool> = const { Cell::new(false) };
}

/// Makes the loads on this thread wait for the parse worker of each file
/// in `take_prefetched_meta`, also when no worker has started it yet, so
/// the loader always takes the worker's metadata (tests). Only for loads
/// with parse workers (`parse_workers_enabled`): with none, a queued job
/// never starts.
#[cfg(test)]
pub(crate) fn set_meta_wait(on: bool) {
    META_WAIT.with(|wait| wait.set(on));
}

/// The preps that loads on this thread took so far (tests).
#[cfg(test)]
pub(crate) fn preps_taken() -> usize {
    PREPS_TAKEN.with(Cell::get)
}

/// Notes a prep that the loader took.
pub(crate) fn note_prep_taken() {
    PREPS_TAKEN.with(|count| count.set(count.get() + 1));
}






/// What a parse worker needs to guess the files that a parsed file
/// references (`queue_references`). The loader takes workers only with its
/// own config (`FilesParser::parse`).
#[derive(Clone, PartialEq, Eq)]
struct PrefetchConfig {
    current_directory: String,
    use_case_sensitive_file_names: bool,
    /// Go `defaultLibraryPath`. Empty when `libReplacement` can move lib
    /// files, so lib references are not guessed.
    default_library_path: String,
}

impl PrefetchConfig {
}




/// What `take_prefetched` gave the loader during one program load.
#[derive(Default)]
pub(crate) struct PrefetchStats {
    /// Worker parses that the loader took.
    taken: usize,
    /// The files that the loader parsed itself, and why: no worker had
    /// started the job (the loader claims it), no job was queued, or the
    /// worker parse was not usable.
    claimed: Vec<String>,
    not_queued: Vec<String>,
    unusable: Vec<String>,
    /// Running worker parses that the loader waited for, and the time.
    waited: usize,
    wait: std::time::Duration,
    /// Worker reads and parses that the loader did not take, and their
    /// text bytes (`PrefetchShared::untaken`).
    untaken: usize,
    untaken_bytes: u64,
    /// Worker metadata that the loader took (`take_prefetched_meta`).
    meta_taken: usize,
    /// Preps that the loader took, and that did not fit (`FilePrep::fits`).
    pub(crate) preps_taken: usize,
    pub(crate) preps_unfit: usize,
    /// Worker parses with no prep when the loader needed it: the worker
    /// still resolved, or it made none.
    preps_missing: usize,
    /// Module names of taken preps with a worker answer, and with none
    /// (another mode than the worker's).
    pub(crate) names_taken: usize,
    pub(crate) names_own: usize,
}

impl PrefetchStats {
}

/// Program loads of this process that started parse workers, and that
/// started none because the host gave every root file from its cache
/// (`FilesParser::parse`). Worker reads (`run_prefetch_worker`), and the
/// reads and their text bytes that no loader took (`PrefetchPool::drop`).
static POOL_LOADS: AtomicUsize = AtomicUsize::new(0);
static CACHED_LOADS: AtomicUsize = AtomicUsize::new(0);
static WORKER_READS: AtomicUsize = AtomicUsize::new(0);
static UNTAKEN_READS: AtomicUsize = AtomicUsize::new(0);
static UNTAKEN_BYTES: AtomicU64 = AtomicU64::new(0);

/// What the parse workers of this process read and parsed, and what of it
/// no program load took. An untaken parse keeps its nodes in the worker's
/// AST arena and its text, so `untaken` should stay 0 for loads that get
/// their files from a cache (`CompilerHost::cached_source_file_refs`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PrefetchCounts {
    /// Program loads that started parse workers.
    pub pool_loads: usize,
    /// Program loads that started none because the host gave every root
    /// file from its cache.
    pub cached_loads: usize,
    /// Files that a worker read (and parsed, when it could).
    pub reads: usize,
    /// Reads that the loader did not take when its pool stopped.
    pub untaken: usize,
    /// The text bytes of the untaken reads.
    pub untaken_bytes: u64,
}

/// The counts of `PrefetchCounts` so far. The workers of a discarded early
/// pool (`start_default_lib_prefetch`) are not in `untaken`.
// PORT: not in Go (parse workers). For tests and debug runs.
pub fn prefetch_counts() -> PrefetchCounts {
    PrefetchCounts {
        pool_loads: POOL_LOADS.load(AtomicOrdering::Relaxed),
        cached_loads: CACHED_LOADS.load(AtomicOrdering::Relaxed),
        reads: WORKER_READS.load(AtomicOrdering::Relaxed),
        untaken: UNTAKEN_READS.load(AtomicOrdering::Relaxed),
        untaken_bytes: UNTAKEN_BYTES.load(AtomicOrdering::Relaxed),
    }
}












/// What `queue_references` reads from a parse. It is taken before the
/// parse goes to the loader, so the loader does not wait for the
/// resolution. The workers only guess with it: the loader checks every
/// file and resolution it takes from them.
#[derive(Debug)]
pub struct FileRefs {
    file_name: String,
    referenced_files: Vec<FileReference>,
    lib_reference_directives: Vec<FileReference>,
    type_reference_directives: Vec<FileReference>,
    import_specifiers: Vec<String>,
    /// The module indicator options that the cached parse read
    /// (`FileRefs::of_kept_parse`): the host gives the parse only for these
    /// options (`fits`). `None`: any options.
    module_indicator_options: Option<ExternalModuleIndicatorOptions>,
}

impl FileRefs {


    /// True when the host probably gives the cached parse for a parse with
    /// module indicator options `options()`. A wrong guess only costs time:
    /// the loader checks each parse.
    fn fits(&self, options: impl FnOnce() -> ExternalModuleIndicatorOptions) -> bool {
        self.module_indicator_options
            .is_none_or(|read| read == options())
    }

}




/// The file whose worker job panics after it starts, for the tests of
/// `RunningJob`. Any thread's load sees it, so a test names a file of its
/// own temp dir.
#[cfg(test)]
pub(crate) static PANIC_IN_JOB: Mutex<Option<String>> = Mutex::new(None);

/// The worker transforms that loads took (`take_prefetched_mapped`), for
/// the tests: the file name, and whether a worker parse of the virtual text
/// came with the transform.
#[cfg(test)]
pub(crate) static MAPPED_TAKEN: Mutex<Vec<(String, bool)>> = Mutex::new(Vec::new());

/// Content-mapped files whose worker job the loader does not claim, for
/// the tests: `take_prefetched_mapped` waits up to 60 s for a worker to
/// start it, so a worker sends its transform. Any thread's load sees it,
/// so a test names files of its own temp dir.
#[cfg(test)]
pub(crate) static MAPPED_WAIT_FOR_WORKER: Mutex<Vec<String>> = Mutex::new(Vec::new());









/// What a parse worker knows of its file for the prep (`FilePrep`) before
/// it resolves: the metadata and the synthetic imports.
struct PrepInput {
    meta: SourceFileMetaData,
    synthetic: SyntheticImports,
}

/// The lookups that Go `cachedvfs` caches (all but `Stat`), in maps that
/// other threads can read. The parse workers of one load share one
/// (`SharedStatCache`). The `tsc -b` host's cached file system keeps its
/// cache in them too (`BuildStatCache`).
#[derive(Default)]
pub struct StatCache {
    file_exists: Mutex<FxHashMap<String, bool>>,
    directory_exists: Mutex<FxHashMap<String, bool>>,
    realpath: Mutex<FxHashMap<String, String>>,
    entries: Mutex<FxHashMap<String, Entries>>,
}




impl StatCache {





}

/// The lookup cache of the `tsc -b` host's file system
/// (`CompilerHost::stat_cache`), shared with the parse workers of its
/// program loads.
///
/// PORT: Go parse tasks share the host's cachedvfs, which lasts for the
/// whole build, and a write does not update it (cachedvfs.go:144). So each
/// lookup that a program makes before the build writes that path stays
/// for the later programs. The parse workers here also resolve guesses:
/// imports with a guessed resolution mode, files that the loader may not
/// load. A lookup of a guess must not stay, because Go does not make it.
/// So the workers read `cached` and keep their own lookups in `load`,
/// which the load clears at its end (`end_load`). What stays in `cached`
/// is what Go caches:
/// - the lookups of the host's file system (the loader and the
///   orchestrator). A lookup that a worker of the load made already is
///   taken from `load` (the build writes nothing during a load);
/// - the lookups of the worker answers that the loader takes
///   (`SharedResolution::lookups`), which the load adds at its end;
/// - the lookups of the config matches that config threads made for the
///   orchestrator (`add`).
#[derive(Default)]
pub struct BuildStatCache {
    /// Go `cachedvfs`: kept for the whole build.
    cached: StatCache,
    /// The lookups of the parse workers of the load that runs now, which
    /// `cached` did not have.
    load: StatCache,
}

impl BuildStatCache {









}

/// Go `cachedvfs` for the parse workers: the lookups that `CachedFs`
/// caches, shared by the workers of one load. Module resolution and import
/// guesses ask for the same paths many times.
///
/// PORT: Go parse tasks share the host's cachedvfs. When the loader's host
/// keeps its cache in a `BuildStatCache` (`CompilerHost::stat_cache`, the
/// `tsc -b` host), the workers use that cache (`host`) in place of `own`:
/// they read what the build has cached, and the host's file system reads
/// what they found.
#[derive(Default)]
struct SharedStatCache {
    own: StatCache,
    host: OnceLock<Arc<BuildStatCache>>,
}

impl SharedStatCache {
}







