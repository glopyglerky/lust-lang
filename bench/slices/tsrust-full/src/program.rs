//! Go `compiler` package (Program, checker pool, diagnostics pipeline) plus
//! the parser-side `ast.SourceFile` fields that the checker reads.
//!
//! The graph (files, parse trees, module resolution) comes from the Go
//! frontend loader (`go_frontend`). This file has the Go `Program` methods
//! as free functions, the Go checker pool and the tsc diagnostics pipeline
//! and non-pretty formatter.
//!
//! Use: `let program = load(config_path); bind_all();` then the diagnostics
//! functions below. `load` installs the program for the process; the
//! loading thread also keeps the frontend program and the checker pool.
//!
//! A multi-program process (watch, language server, tests) loads program
//! versions with `try_load_version` and `update_program_version`, reads one
//! inside `core::enter_program`, and frees its checker pool with
//! `release_program`. The loading thread keeps the frontend program and the
//! checker pool of each version, by program id.

use crate::frontend::parser::ExternalModuleIndicatorOptions;
use crate::frontend::tspath;
use crate::gostd::{context};
use crate::prelude::*;
use std::borrow::Cow;
use std::ops::Deref;
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

mod go_frontend;
pub mod ls_program;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Go `ast.PragmaArgument`.
#[derive(Clone, Debug, Default)]
pub struct PragmaArgument {
    pub name: String,
    pub value: String,
    pub range: TextRange,
}


/// Go `ast.CheckJsDirective`.
#[derive(Clone, Copy, Debug, Default)]
pub struct CheckJsDirective {
    pub enabled: bool,
    pub range: TextRange,
}

/// Go `ast.FileReference`.
#[derive(Clone, Debug, Default)]
pub struct FileReference {
    pub range: TextRange,
    pub file_name: String,
    pub resolution_mode: ResolutionMode,
    pub preserve: bool,
}

/// Go `ast.CommentDirective`.
#[derive(Clone, Copy, Debug, Default)]
pub struct CommentDirective {
    pub loc: TextRange,
    pub kind: CommentDirectiveKind,
}

/// Go `ast.SourceFileMetaData`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SourceFileMetaData {
    pub package_json_type: String,
    pub package_json_directory: String,
    pub implied_node_format: ModuleKind,
}

/// Go `module.PackageId`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct PackageId {
    pub name: String,
    pub sub_module_name: String,
    pub version: String,
    pub peer_dependencies: String,
}







/// A parse list that a `SourceFileInfo` keeps (lsshells M3b): borrowed for
/// good from the parse that a static publish keeps (or leaked), or owned by
/// the `GoFile` of a freeable file version. It derefs to the list.
pub enum KeptData<T: ?Sized + 'static> {
    Borrowed(&'static T),
    Owned(Box<T>),
}



/// Go `ast.SourceFile` fields that need the installed tree.
pub struct LateSourceFileInfo {
    pub file_index: usize,
    pub external_module_indicator: Node,
    // PERF: like the diagnostic lists, `reparsed_clones` and `jsdoc_cache`
    // of a static file borrow the kept parse; a freeable file version owns
    // copies. The JSDoc cache has one list per host node, so a copy costs
    // one allocation per entry.
    pub reparsed_clones: KeptData<[Node]>,
    pub imports: Vec<Node>,
    pub module_augmentations: Vec<Node>,
    pub ambient_module_names: Vec<String>,
    pub uses_uri_style_node_core_modules: Tristate,
    /// Go `SourceFile.jsdocCache`: parsed JSDoc nodes by host node.
    pub jsdoc_cache: KeptData<FxHashMap<Node, Vec<Node>>>,
    post_bind: OnceLock<PostBindInfo>,
}


/// Go `ast.SourceFile` fields that the binder sets but that live on the
/// SourceFile in Go.
pub struct PostBindInfo {
    pub common_js_module_indicator: Node,
}

static NOT_BOUND: PostBindInfo = PostBindInfo {
    common_js_module_indicator: Node::NIL,
};


/// The threads that join the workers of the checker pools that
/// `shut_down_in_background` stopped, except those that had ended at the
/// last stop (`wait_for_background_releases`).
static BACKGROUND_STOPS: Mutex<Vec<std::thread::JoinHandle<()>>> = Mutex::new(Vec::new());







/// The most threads of an emit pool.
const MAX_EMIT_THREADS: usize = 32;






/// PORT: not in Go (perf). The twin of a checker worker: a thread that
/// prints the JS and d.ts parts whose transforms ran on the checker
/// (`program_emit`), so the checker can go on with its next file. The
/// transforms call the emit resolver, so they stay on the checker, and each
/// checker's d.ts text depends on its own check. The prints need no checker:
/// Go `emitJSFile` gives its printer no handlers, and the only print handler
/// of Go `emitDeclarationFile` maps declaration map positions through the
/// span map of a content-mapped source file. Go runs each file's emit on its
/// own goroutine and locks the checker only for each resolver call.
///
/// The twin shares the checker's synthetic chunk numbers
/// (`share_synthetic_chunks`), so a handle names the same node on both
/// threads, and each job brings the nodes that its print reads
/// (`PrintPack`). The pool threads cannot print these trees: their chunk
/// numbers and their thread-local maps keyed by node are their own. The
/// twin runs its jobs in the order they are sent. It is made on the first
/// job of its checker, and it stops with the checker pool
/// (`CheckerPool::stop`).
#[cfg(not(target_family = "wasm"))]
struct DtsTwin {
    queue: std::sync::mpsc::Sender<Job>,
    thread: std::thread::JoinHandle<()>,
    /// Set by `stop_dts_twin`: the twin frees its synthetic nodes when its
    /// queue closes. Else it ends with the process and leaks them, like its
    /// checker.
    released: Arc<std::sync::atomic::AtomicBool>,
}

/// The jobs sent to d.ts twins in this process (`dts_twin_job_count`).
static DTS_TWIN_JOBS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);



/// wasm has no d.ts twins: with no emit pool (`emit_thread_count`), no
/// file's emit is split.
#[cfg(target_family = "wasm")]
pub fn send_dts_twin_job<R: Send + 'static>(
    _f: impl FnOnce() -> R + Send + 'static,
) -> EmitPoolJob<R> {
    unreachable!("d.ts twin job on wasm")
}


/// The number of jobs sent to d.ts twins in this process so far. Tests use
/// it to see that the twins ran.
pub fn dts_twin_job_count() -> usize {
    DTS_TWIN_JOBS.load(std::sync::atomic::Ordering::Relaxed)
}


/// wasm has no emit pool (`emit_thread_count`, `emit_pool_enabled`).
#[cfg(target_family = "wasm")]
pub fn send_emit_pool_jobs<R: Send + 'static>(
    _jobs: Vec<impl FnOnce() -> R + Send + 'static>,
) -> Vec<EmitPoolJob<R>> {
    unreachable!("emit pool job on wasm")
}


/// wasm has no emit pool.
#[cfg(target_family = "wasm")]
pub fn emit_pool_job_count() -> usize {
    0
}

/// Work for one checker worker. It runs on the worker thread, where
/// `with_checker_at` reaches that worker's checker.
type Job = Box<dyn FnOnce() + Send>;



/// wasm: the result of a job, which ran when it was sent
/// (`send_thread_job`). With no channel, the module has no channel code
/// for each result type.
#[cfg(target_family = "wasm")]
pub struct JobReceiver<R>(R);





/// Gives the files of `VersionTables::file_by_path` their slots: a program
/// file its position in the file order, and any other file the next slot
/// after the program files.
struct SlotsBuilder<'a> {
    source_file_order: &'a [usize],
    /// The slot of each file id, made on first use.
    slots: Option<FxHashMap<usize, usize>>,
    /// `VersionTables::other_files`.
    other_files: Vec<usize>,
}

impl<'a> SlotsBuilder<'a> {
    fn new(source_file_order: &'a [usize]) -> Self {
        SlotsBuilder {
            source_file_order,
            slots: None,
            other_files: Vec::new(),
        }
    }

    /// The slot of file `file`.
    fn slot_of(&mut self, file: usize) -> usize {
        let order = self.source_file_order;
        let slots = self.slots.get_or_insert_with(|| {
            order
                .iter()
                .enumerate()
                .map(|(slot, &file)| (file, slot))
                .collect()
        });
        *slots.entry(file).or_insert_with(|| {
            self.other_files.push(file);
            order.len() + self.other_files.len() - 1
        })
    }
}



/// True when `GOPORT_KEEP_VERSION_TABLES=1`: a released program version
/// keeps its tables, as before lsshells M2a (for A/B runs, and as a field
/// fallback). Read once.
fn keep_version_tables() -> bool {
    static KEEP: OnceLock<bool> = OnceLock::new();
    *KEEP.get_or_init(|| std::env::var("GOPORT_KEEP_VERSION_TABLES").is_ok_and(|v| v == "1"))
}






















/// Go `SourceFile.IsDefaultLibrary` (read through the program) and
/// `SourceFile.Metadata` of one program file.
#[derive(Clone, Debug, PartialEq)]
struct FileProgramMeta {
    meta_data: SourceFileMetaData,
    is_default_library: bool,
}







// ---------------------------------------------------------------------------
// Alias resolver programs (Go ls/autoimport/aliasresolver.go)
// ---------------------------------------------------------------------------









// ---------------------------------------------------------------------------
// Loading
// ---------------------------------------------------------------------------







/// The generation of the binder lineage (`LineageSeen::generation`), so a
/// checker whose copy is current does not lock the lineage
/// (`catch_up_checker`).
static LINEAGE_GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);






// Go: compiler/program.go:574 BindSourceFiles
// PORT: Go binds files in parallel into per-file symbol tables. Here every
// file binds into the shared `LINEAGE` arena, and the program's binder
// symbols (`bound_symbols`, in its `VersionTables`) are a copy of it after
// the program files are bound. `Checker::new` uses the same initializer,
// so the first of the two to run binds. Files bind in parallel, each into
// its own arena (`bind_files_parallel`), and join the lineage in file order
// with the ids a serial bind gives. With `--singleThreaded` they bind
// last-queued-first (`bind_files_last_queued_first`). A file that an
// earlier program version bound is not bound again.





/// From this many files to bind on, a load binds as a large load on
/// `WIDE_CORES` physical cores (`bind_thread_count`). Query (`-p`, 186
/// files) stays below it: there 8 bind threads were faster than 16.
const WIDE_BIND_FILES: usize = 300;

/// A program load with at least this many root tasks (root files, `lib`
/// entries and the automatic type directive task) is large.
// PERF (perf9 round 3, effect R3-E1): root tasks are query 27, hono 190,
// elysia 241, zod 324 and effect 459. At 16 threads, 7 parse workers
// parsed effect 12 ms and zod 9 ms faster than 4, and hono in the same
// time. With glibc malloc, query gained nothing from more parse threads
// (lib.dom bounds its parse), and its peak RSS was near the 1.15x rule.
// With jemalloc, a load that is not large parses on 8 threads too
// (`ThreadBudget::one_program`).
const LARGE_LOAD_ROOT_TASKS: usize = 128;

thread_local! {
    /// Whether the last program load on this thread was large
    /// (`note_program_load`). The bind of that program reads it.
    static LARGE_LOAD: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Records whether a program load on this thread with `root_tasks` root
/// tasks is large, and returns it. A large load gets more parse threads
/// (`ThreadBudget::parse_threads`) and bind threads
/// (`ThreadBudget::bind_threads`). Call it when the load starts.
pub(crate) fn note_program_load(root_tasks: usize) -> bool {
    let large = root_tasks >= LARGE_LOAD_ROOT_TASKS;
    LARGE_LOAD.set(large);
    large
}


/// From this many physical cores on (`wide_cores`), a large program load
/// of a one-program process parses and binds on this many threads
/// (`ThreadBudget::one_program`).
const WIDE_CORES: usize = 16;





/// The most parse threads (the loading thread included) and bind threads
/// of a program load, and the glibc malloc arenas of the process, in one
/// budget. A large program load (`note_program_load`) has its own limits.
///
/// glibc gives each thread that mallocs its own arena until `arena_max`
/// arenas exist. A new thread first takes the arena of a thread that
/// ended, if there is one. When `arena_max` arenas exist, a later thread
/// shares the arena of another thread, and when both malloc at once they
/// wait on its lock. Each arena in use also adds peak RSS, because freed
/// per-file data stays in it. The parse mallocs most (a quarter of the
/// parse thread cycles), so the parse threads must fit the arenas: with 7
/// arenas at 16 cores, the 7 parse workers of effect made 3,763 futex waits
/// (19 at 4 cores). The bind threads malloc little (about 5% of their
/// cycles), so they can share arenas: 8 bind threads bind effect in the
/// same time with 7 arenas as with 16. In the check, the checkers and the
/// loading thread malloc. Main, and in `tsgo` the signal and text hash
/// threads, hold an arena too.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThreadBudget {
    /// The most parse threads, the loading thread included.
    pub parse: usize,
    /// The most parse threads of a large program load (see `widen`).
    pub parse_large: usize,
    /// The most bind threads of a large program load (see `widen`).
    pub bind: usize,
    /// The most bind threads of other program loads.
    pub bind_small: usize,
    /// `glibc.malloc.arena_max`.
    pub arena_max: usize,
    /// When the process may run on `WIDE_CORES` physical cores
    /// (`wide_cores`), a large load parses and binds on `WIDE_CORES` threads
    /// instead of `parse_large` and `bind`.
    pub widen: bool,
}

static THREAD_BUDGET: OnceLock<ThreadBudget> = OnceLock::new();

/// Go's default checker count (`checker_count`).
const DEFAULT_CHECKERS: usize = 4;

impl ThreadBudget {
    /// A process that loads several programs at once (`goport_build`: about
    /// 20 threads per program), and the counts of a process that installs
    /// no budget.
    pub const WIDE: ThreadBudget = ThreadBudget {
        parse: 8,
        parse_large: 8,
        bind: 8,
        bind_small: 8,
        arena_max: 16,
        widen: false,
    };





    /// Makes this the budget of the program loads of this process. The
    /// first install wins.
    pub fn install(self) {
        let _ = THREAD_BUDGET.set(self);
    }

    /// The installed budget, or `WIDE`.
    pub fn current() -> Self {
        THREAD_BUDGET.get().copied().unwrap_or(Self::WIDE)
    }

}







// ---------------------------------------------------------------------------
// Resolution modes (Go compiler/fileloader.go)
// ---------------------------------------------------------------------------

// Go: compiler/fileloader.go:1041 getDefaultResolutionModeForFile

// Go: compiler/fileloader.go:1049 getModeForUsageLocation

// Go: compiler/fileloader.go:1081 importSyntaxAffectsModuleResolution

// Go: compiler/fileloader.go:1087 getEmitSyntaxForUsageLocationWorker

// ---------------------------------------------------------------------------
// Program methods (Go compiler/program.go). The program is the current
// `prog()`; these are free functions.
// PORT: Go `projectReferenceFileMapper.getCompilerOptionsForFile` returns the
// program options when there are no project references, which is always the
// case here.
// ---------------------------------------------------------------------------


// Go: compiler/program.go:129 FileExists
// Go: ls/autoimport/aliasresolver.go:163 FileExists (unimplemented)


// Go: compiler/program.go:134 GetCurrentDirectory

// Go: compiler/program.go:231 UseCaseSensitiveFileNames

// Go: compiler/program.go:235 UsesUriStyleNodeCoreModules
// Go never assigns the program field (only UpdateProgram copies it), so the
// value is always unknown.
pub fn uses_uri_style_node_core_modules() -> Tristate {
    Tristate::Unknown
}

// Go: compiler/program.go:189 GetProjectReferenceFromSource
// PORT: the Go frontend program has the port.

// Go: compiler/program.go:194 IsSourceFromProjectReference

// Go: compiler/program.go:198 GetProjectReferenceFromOutputDts
// PORT: see `get_project_reference_from_source`.

// Go: compiler/program.go:206 GetRedirectForResolution
// PORT: see `get_project_reference_from_source`.

// Go: compiler/projectreferencefilemapper.go:80 getCompilerOptionsForFile
// Go: module/resolver.go:138 GetCompilerOptionsWithRedirect
// Go: compiler/program.go:1739 GetSourceFileMetaData
// Runs `f` with the options of the project reference that owns the file
// (else the root options) and the metadata of the file (the default
// metadata when it is not a program file). The per-file mode queries below
// use it for each import, so neither value is copied.

// Go: compiler/program.go:215 GetResolvedProjectReferences
// PORT: see `get_project_reference_from_source`. A reference that did not
// load is None (Go nil).

// Go: compiler/program.go:2300 GetSymlinkCache
// PORT: the Go frontend program has the port, and this is a copy of its
// value, built on first use (`go_frontend::known_symlinks`).

// Go: compiler/program.go:181 GetSourceOfProjectReferenceIfOutputIncluded

















// Go: compiler/program.go:2120 ExplainFiles

// Go: compiler/program.go:511 SourceFiles

// Go: compiler/program.go:513 Options

// Go: compiler/program.go:528 ContentMapperExtensions (#4712)
// PORT: the Go frontend program copies the extensions of its command line
// (`GoSharedState`), so checker threads can read them. An alias resolver
// program has none.



// Go: compiler/program.go:532 GetConfigFileParsingDiagnostics
// PORT: an alias resolver program has no config, so its list is empty. Go
// has no such method on the alias resolver.

// Go: compiler/program.go:570 SingleThreaded



// Go: compiler/program.go:623 GetResolvedModule
// PERF: the stored resolution is shared, not copied.

// Go: compiler/program.go:632 GetResolvedModuleFromModuleSpecifier

// Go: compiler/program.go:640 GetResolvedModules
// Go: ls/autoimport/aliasresolver.go:135 GetResolvedModules (nil)
// PORT: only an alias resolver program reads this map, and it is empty. The
// Go frontend program keeps its map in `GoSharedState` (`get_packages_map`).

// Go: compiler/program.go:1739 GetSourceFileMetaData

// Go: compiler/program.go:1743 GetEmitModuleFormatOfFile

// Go: compiler/program.go:1747 GetEmitSyntaxForUsageLocation

// Go: compiler/program.go:1751 GetImpliedNodeFormatForEmit

// Go: compiler/program.go:1755 GetModeForUsageLocation

// Go: compiler/program.go:1776 GetDefaultResolutionModeForFile

// Go: compiler/program.go:1780 IsSourceFileDefaultLibrary

// Go: compiler/program.go:1799 CommonSourceDirectory
// PORT: the Go frontend sets it when it builds the program
// (`go_frontend::common_source_directory_of`).

// Go: compiler/program.go:2195 IsSourceFileFromExternalLibrary

// Go: compiler/program.go:1390 IsEmitBlocked

// Go: compiler/program.go:2210 SourceFileMayBeEmitted

// Go: compiler/emitter.go:493 sourceFileMayBeEmitted

// Go: compiler/emitter.go:504-521 (the JSON file part of sourceFileMayBeEmitted)
// PORT: its own function, so the tests below can run it without a loaded
// program. Like `frontend::compiler::source_file_may_be_emitted`, it uses the
// Go ports in `frontend::outputpaths` and `frontend::tspath`. Go result to
// keep: `GetNormalizedAbsolutePath` removes the trailing separator of the
// common directory (except for a root), so a file under it gets a rooted
// output path that is never its own path, even when outDir is the common
// directory.

// Go: compiler/program.go:2071 GetSourceFile

// Go: compiler/program.go:2091 GetSourceFileByPath

/// A memo of `get_source_file_for_resolved_module` by file name, for the
/// program with id `program`.
struct ResolvedModuleFiles {
    program: u32,
    files: FxHashMap<String, Node>,
}


// Go: compiler/program.go:2076 GetSourceFileForResolvedModule
// PORT: the answer is memoized per thread and program. The files, their
// paths and the redirects of a program do not change after load, so a name
// always gives the same file. The checker asks again on each alias and
// default-import check, and each lookup canonicalizes the path. Another
// program version can give another file (an edited file has a new id), so
// the memo is emptied when the current program changes.

// Go: compiler/program.go:173 GetRedirectTargets

// Go: compiler/program.go:242 GetSourceFileFromReference
// PORT: the Go frontend program has the port, and its answers for the
// preserved references are copied. Go has no such method on the alias
// resolver.

// Go: outputpaths/outputpaths.go:47 GetOutputPathsFor, called by
// compiler/emitHost.go:94 emitHost.GetOutputPathsFor and Program.Emit with
// the program options.
// PORT: Go reads three fields of the source file (#4712: the content
// mapper). It is named for the emit host method so it does not clash with
// the frontend `get_output_paths_for` in the frontend prelude.

// Go: compiler/program.go:2199 GetJSXRuntimeImportSpecifier
// Go: compiler/fileloader.go:550 (the value the loader records)
// PORT: the Go frontend loader records the value and its synthetic import
// (Go `createSyntheticImport`).

// Go: compiler/program.go:2206 GetImportHelpersImportSpecifier
// PORT: the Go frontend loader records the synthetic imports.

// Go: compiler/program.go:650 GetPackagesMap
// PORT: made once per program version, as Go `packagesMapOnce`, and kept
// in its tables. The checker copies it (`Checker::get_packages_map`).

// Go: compiler/program.go:652 (the packagesMapOnce func)
// Go: checker/utilities.go:1722 getPackagesMap (the same body)
// PORT: Go ranges over `GetResolvedModules()`, the program's own map. With
// the Go frontend that is the version's map in `GoSharedState`, borrowed,
// so no owned copy of every resolution stays with the version. An alias
// resolver program (no resolutions) uses `get_resolved_modules`. The result
// does not depend on the order: each value is an OR.

// ---------------------------------------------------------------------------
// Checker pool (Go compiler/checkerpool.go)
// PORT: Go runs one goroutine task per checker on a work group. A Rust
// checker is not Send (it holds Rc and thread-local state), so each checker
// is made on its own worker thread and stays there. The loading thread
// sends jobs to the workers and waits for the results, which it merges in
// file order. Each checker sees only its own files, in file order, so the
// results match the Go grouping and do not depend on thread timing.
// ---------------------------------------------------------------------------





// Go: compiler/checkerpool.go:307 newCheckerPoolWithTracing (the count)

// Go: compiler/checkerpool.go:365 createCheckers
// PORT: binding runs first on this thread, so no worker binds and every
// worker starts from the same bound program and thread-local state.
// The file associations (#4313) come from the Go frontend program
// (`ls_program::get_checker_associations`), whose files are in
// `source_file_order` order. An alias resolver program has no frontend, so
// it keeps the round-robin order; Go gives it no checker pool.

// Go: compiler/checkerpool.go:365 createCheckers (one `checker.NewChecker`)



/// wasm has one thread: makes the `count` checkers on the loading thread,
/// each with its own `WorkerIds`, as a worker would make it.
#[cfg(target_family = "wasm")]
fn start_checkers(count: usize) -> CheckerPool {
    let checkers = (0..count)
        .map(|index| {
            let mut ids = WorkerIds(id_seed().into());
            let checker = ids.run(|| new_pool_checker(index, count));
            Some((checker, ids))
        })
        .collect();
    CheckerPool {
        checkers,
        carry_from: symbol_id_carry_start(count),
    }
}

/// wasm: the ids of one checker of the inline pool. A native checker
/// worker starts from a copy of the loading thread's ids (`WorkerSeed`) and
/// counts on its own from then on, so the ids that one checker gives do not
/// move the ids of another (some names hold symbol ids, and members with
/// no declaration sort by name). The checkers share the loading thread's
/// synthetic nodes and lazy JSDoc: those keep each node's handle, which
/// thread-local caches key on.
#[cfg(target_family = "wasm")]
struct WorkerIds(crate::ast::IdState);

#[cfg(target_family = "wasm")]
impl WorkerIds {
    /// Runs `f` with these ids as the thread's, and keeps the ids that `f`
    /// leaves. A panic aborts on wasm, so nothing restores the ids after
    /// one.
    fn run<R>(&mut self, f: impl FnOnce() -> R) -> R {
        crate::ast::swap_id_state(&mut self.0);
        let result = f();
        crate::ast::swap_id_state(&mut self.0);
        result
    }
}



/// wasm: every job already ran when it was sent (`send_thread_job`), so
/// each value drops at once.
#[cfg(target_family = "wasm")]
pub fn send_checker_barrier<T: Send + 'static>(signal: impl Fn() -> T) -> usize {
    let id = prog().id;
    POOLS.with(|pools| {
        let pools = pools.borrow();
        let Some(pool) = pools.get(&id) else {
            return 0;
        };
        for _ in &pool.checkers {
            drop(signal());
        }
        pool.checkers.len()
    })
}




/// wasm: runs `f` now, on the loading thread, as the worker of checker
/// `index` would: with the worker's ids (`WorkerIds`), and with the checker
/// as this thread's worker checker (`with_checker_at`). The returned
/// `JobReceiver` holds the result. A panic aborts on wasm, so there is no
/// panic payload to keep.
#[cfg(target_family = "wasm")]
fn send_thread_job<R: Send + 'static>(
    index: usize,
    f: impl FnOnce() -> R + Send + 'static,
) -> JobReceiver<R> {
    assert!(
        worker_index().is_none(),
        "checker job sent from a checker job"
    );
    let id = prog().id;
    // The pool borrow ends before the job runs: a job reads the pool
    // (`checker_index_for_file`).
    let (checker, mut ids) = with_pool(|pool| pool.checkers[index].take().expect("checker in use"));
    let (result, checker) = ids.run(|| {
        WORKER_CHECKER.with(|slot| *slot.borrow_mut() = Some(checker));
        WORKER_INDEX.with(|slot| slot.set(Some(index)));
        let result = f();
        WORKER_INDEX.with(|slot| slot.set(None));
        let checker = WORKER_CHECKER
            .with(|slot| slot.borrow_mut().take())
            .expect("worker checker");
        (result, checker)
    });
    POOLS.with(|pools| {
        if let Some(pool) = pools.borrow_mut().get_mut(&id) {
            pool.checkers[index] = Some((checker, ids));
        }
    });
    JobReceiver(result)
}




/// The jemalloc arena of this thread (`thread.arena`), or None in a build
/// without jemalloc.
fn jemalloc_thread_arena() -> Option<u32> {
    #[cfg(all(feature = "jemalloc", not(windows)))]
    {
        use tikv_jemalloc_ctl::{Access, AsName};
        b"thread.arena\0".name().read().ok()
    }
    #[cfg(not(all(feature = "jemalloc", not(windows))))]
    None
}

/// Makes this thread allocate in jemalloc arena `arena` (`thread.arena`,
/// from `jemalloc_thread_arena`). A failure keeps its arena.
fn set_jemalloc_thread_arena(arena: u32) {
    #[cfg(all(feature = "jemalloc", not(windows)))]
    {
        use tikv_jemalloc_ctl::{Access, AsName};
        let _ = b"thread.arena\0".name().write(arena);
    }
    #[cfg(not(all(feature = "jemalloc", not(windows))))]
    let _ = arena;
}



// Go: compiler/checkerpool.go:344 getCheckerForFileNonExclusive
// PORT: Go returns the checker and a release function. Here the checker is
// lent to `f` for the call, on the checker's own thread.




// PORT: replaces EmitResolver.checkerMu. Lends checker `index` to `f`. Only
// the worker thread of that checker can do this; the checker is borrowed
// for the call, so `f` must not ask for it again.

// Go: compiler/checkerpool.go:449 forEachCheckerParallel

// Go: compiler/checkerpool.go:462 GetGlobalDiagnostics

// Go: compiler/checkerpool.go:474 forEachCheckerGroupDo
// PORT: returns the results of `cb` by file position instead of passing the
// position to `cb`. A file with no result keeps an empty list.




// ---------------------------------------------------------------------------
// Diagnostics (Go compiler/program.go)
// ---------------------------------------------------------------------------

// Go: compiler/program.go:667 collectDiagnostics
// PORT: the per-file work runs serially (see the checker pool note).

// Go: compiler/program.go:695 collectCheckerDiagnostics

// Go: compiler/program.go:708 filterAndSortDiagnostics (#4712)

// Go: compiler/program.go:720 collectCheckerDiagnosticsFromFiles

// Go: compiler/program.go:743 GetSyntacticDiagnostics

// Go: compiler/program.go:762 getAdditionalJSSyntacticDiagnostics

// Go: compiler/program.go:787 GetBindDiagnostics
// PORT: Go binds one file when given one. The Rust binder binds every file
// into one shared arena, so this always binds all files.

// Go: compiler/program.go:798 GetSemanticDiagnostics
// PORT: the compile path has no context; Go tsc passes context.Background()
// (execute/tsc/emit.go:75). The same holds for the two functions below.

// Go: compiler/program.go:804 GetSemanticDiagnosticsForIncremental
// GetSemanticDiagnosticsForIncremental includes newly discovered globals in each
// file's cached diagnostics and leaves noEmit filtering to the builder.




// Go: compiler/program.go:815 GetSuggestionDiagnostics

// Go: compiler/program.go:819 GetProgramDiagnostics
// PORT: an alias resolver program has no frontend program, so its list is
// empty. Go has no such method on the alias resolver.

// Go: compiler/program.go:840 GetIncludeProcessorDiagnostics
// PORT: an alias resolver program has no include diagnostics.

// Go: compiler/program.go:848 SkipTypeChecking

// Go: compiler/program.go:856 canIncludeBindAndCheckDiagnostics

// Go: compiler/program.go:1455 GetGlobalDiagnostics

// Go: compiler/program.go:1467 GetDeclarationDiagnostics
// PORT: each file's work runs on the thread of the file's checker, where its
// emit resolver can reach that checker. Files of different checkers run in
// parallel; the results merge in file order.


// Go: compiler/program.go:1618 getDeclarationDiagnosticsForFile


// Go: compiler/emitter.go:553 getSourceFilesToEmit
// PORT: Go takes a `SourceFileMayBeEmittedHost`; the program functions are that host.
// Go `Program.getSourceFilesToEmit` caches the result for nil targets and no
// force; this computes it each time. Go nil `targetSourceFiles` is `None`.

// Go: compiler/emitter.go:562 isSourceFileNotJson

// Go: compiler/emitter.go:566 getDeclarationDiagnostics
// PORT: renamed from Go `getDeclarationDiagnostics`, because the exported
// `GetDeclarationDiagnostics` above already has the snake name.

// Go: compiler/emitHost.go:33 emitHost
// NOTE: emitHost operations must be thread-safe

// Go: compiler/emitHost.go:38 newEmitHost
// PORT: Go gets the file's checker and a `done` func that releases it. The
// checker is lent only for the `GetEmitResolver` call here; the resolver
// must reach its checker itself. Call it on the thread of the file's checker.




// PORT: Go `emitHost` also implements `modulespecifiers.ModuleSpecifierGenerationHost`
// (GetModeForUsageLocation, GetResolvedModuleFromModuleSpecifier,
// GetDefaultResolutionModeForFile, FileExists, GetGlobalTypingsCacheLocation,
// GetNearestAncestorDirectoryWithPackageJson, GetPackageJsonInfo,
// GetSourceOfProjectReferenceIfOutputIncluded, GetProjectReferenceFromSource,
// GetRedirectTargets, GetSymlinkCache, ResolveModuleName). That interface is
// not ported, so those methods are left out until it is.


// Go: compiler/program.go:1471 FilterNoEmitSemanticDiagnostics

// Go: compiler/program.go:1480 getSemanticDiagnosticsWithChecker

// Go: compiler/program.go:1490 getBindAndCheckDiagnosticsWithChecker
// getBindAndCheckDiagnosticsWithChecker gets semantic diagnostics for a single file using a
// caller-provided checker, including bind diagnostics, checker diagnostics, and handling
// of @ts-ignore/@ts-expect-error directives.

// Go: compiler/program.go:1543 applyContentMapperDiagnosticDirectives (#4712)

// Go: compiler/program.go:1579 getDiagnosticsWithPrecedingDirectives
// PORT: Go returns a map by line; its iteration order is random and the
// caller sorts the result later. A BTreeMap gives a fixed order.

// Go: compiler/program.go:1634 getSuggestionDiagnosticsWithChecker

// Go: compiler/program.go:1642 isCommentOrBlankLine
fn is_comment_or_blank_line(text: &str, mut pos: usize) -> bool {
    let text = text.as_bytes();
    while pos < text.len() && (text[pos] == b' ' || text[pos] == b'\t') {
        pos += 1;
    }
    pos == text.len()
        || pos < text.len() && (text[pos] == b'\r' || text[pos] == b'\n')
        || pos + 1 < text.len() && text[pos] == b'/' && text[pos + 1] == b'/'
}

// Go: compiler/program.go:1651 SortAndDeduplicateDiagnostics
// PERF (startexit1): Go sorts pointers, and each compare reads the two file
// names. The port sorts the indexes of the diagnostics with each file name
// ranked once (`DiagnosticPaths`), so a compare reads no file name and moves
// no diagnostic, and the merge moves each diagnostic where it cloned each
// one. The sort makes the same compares and swaps on the indexes as on the
// diagnostics, so the order is Go's (`gostd::slices`, `by_index`). midway
// (10,572 diagnostic lines) sorted 7x slower than Go (perfmeas1/phase).

/// The Go path of each diagnostic of a list (`get_diagnostic_path`), read
/// once per file, as an index into the distinct paths, and the rank of each
/// distinct path in Go byte order (`compare_go_bytes`; paths with the same
/// Go bytes have the same rank). For `sort_and_deduplicate_diagnostics`.
struct DiagnosticPaths {
    /// Per diagnostic: its path, as an index into `names` and `ranks`.
    path: Vec<u32>,
    names: Vec<&'static str>,
    ranks: Vec<u32>,
}

impl DiagnosticPaths {

    /// The rank of the path of diagnostic `i`.
    fn rank(&self, i: usize) -> u32 {
        self.ranks[self.path[i] as usize]
    }

    /// Whether diagnostics `a` and `b` have the same path (Go `==` of the
    /// paths in `equal_diagnostics_no_related_info`).
    fn same(&self, a: usize, b: usize) -> bool {
        self.names[self.path[a] as usize] == self.names[self.path[b] as usize]
    }
}

// Go: compiler/program.go:1659 compactAndMergeRelatedInfos
// Remove duplicate diagnostics and, for sequences of diagnostics that differ only by related information,
// create a single diagnostic with sorted and deduplicated related information.
// PORT: `order` is the sorted order of `diagnostics` (see
// `sort_and_deduplicate_diagnostics`). Each kept diagnostic moves to the
// result; Go keeps the pointer, or a clone with the merged related
// information.

// Go: compiler/program.go:1690 LineCount

// Go: compiler/program.go:1698 IdentifierCount

// Go: compiler/program.go:1706 SymbolCount
// PORT: an unbound file (the program had syntactic errors) has the Go zero
// value.

// Go: compiler/program.go:1719 TypeCount

// Go: compiler/program.go:1727 InstantiationCount

// Go: compiler/program.go:2010 GetDiagnosticsOfAnyProgram
// PORT: Go calls `program.GetGlobalDiagnostics` and
// `program.GetDeclarationDiagnostics` directly. They are callbacks here so a
// caller can guard them the same way as the bind and semantic callbacks.
// Go nil `files` is `None` (#4699). `is_compiler_program` is the Go type
// assertion `program.(*Program)`: true for a plain program, false for an
// incremental one.

// Go: compiler/program.go plainJSErrors
// PORT: built on each call from the generated message statics (a static set
// cannot read them at compile time). It is only used for plain JS files.

// ---------------------------------------------------------------------------
// Output (Go diagnosticwriter/diagnosticwriter.go, non-pretty)
// ---------------------------------------------------------------------------

// Go: diagnosticwriter/diagnosticwriter.go:571 WriteFormatDiagnostic
// PORT: Go writes to an io.Writer; this returns the text.
// PORT: Go wraps the diagnostic in `ASTDiagnostic`, whose `File` and `Pos`
// go through `resolve` (#4712): see `resolve_diagnostic_location`.

// Go: diagnosticwriter/diagnosticwriter.go:87 resolvedLocation (#4712)
// resolvedLocation describes how a diagnostic on a content-mapped file should be reported.
struct ResolvedLocation {
    loc: TextRange,
    use_original: bool, // render against the file's original, untransformed text
    synthesized: bool,  // the range is in virtual code with no corresponding original location
}

// Go: diagnosticwriter/diagnosticwriter.go:97 (*ASTDiagnostic).resolve (#4712)
// resolve determines where and against which text a diagnostic should be reported. A content mapper's
// own diagnostics already carry original ranges. A compiler diagnostic on a content-mapped file has its
// virtual range mapped back to the original; if it falls entirely within synthesized code, there is no
// original location, so it is shown against the virtual text and flagged as synthesized.


// Go: diagnosticwriter/diagnosticwriter.go:380 diagnosticPrefix (#4712)
// diagnosticPrefix returns the prefix shown before a diagnostic's code, e.g. "TS" for compiler
// diagnostics or a content mapper's custom source for its diagnostics.

// Go: diagnosticwriter/diagnosticwriter.go:565 WriteFormatDiagnostics

// Go: diagnosticwriter/diagnosticwriter.go:358 WriteFlattenedDiagnosticMessage
// PORT: this writer has no Go `FormattingOptions`, so the locale is
// Go `locale.Default` and the text is English (see execute/tsc/diagnostics.rs
// `write_format_diagnostic`).

// Go: diagnosticwriter/diagnosticwriter.go:146 (*ASTDiagnostic).MessageChain
// PORT: Go wraps each chain entry in `ASTDiagnostic`; the entries are the
// diagnostics themselves here.

// Go: diagnosticwriter/diagnosticwriter.go:366 flattenDiagnosticMessageChain

// Emit support: run work on a file's checker thread without holding the checker.






#[cfg(test)]
mod tests {
    use super::*;

    // startexit1: the index sort with ranked file names gives the order and
    // the merges of Go's `SortAndDeduplicateDiagnostics` (compiler/program.go:
    // 1651): a sort of the diagnostics with `ast.CompareDiagnostics` and
    // `compactAndMergeRelatedInfos`, as the port did before (`reference`).
    // The files include two parses with one name, so diagnostics that compare
    // equal but keep different file nodes show the tie order of the sort.
    #[test]
    fn sort_and_deduplicate_matches_the_value_sort() {
        use crate::frontend::parser::{SourceFileParseOptions, parse_source_file};
        fn reference(mut diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
            crate::gostd::slices::sort_func(&mut diagnostics, compare_diagnostics);
            let mut result = Vec::new();
            let mut i = 0;
            while i < diagnostics.len() {
                let d = &diagnostics[i];
                let mut n = 1;
                while i + n < diagnostics.len()
                    && equal_diagnostics_no_related_info(d, &diagnostics[i + n])
                {
                    n += 1;
                }
                let mut merged = d.clone();
                if n > 1 {
                    let mut related: Vec<Diagnostic> = diagnostics[i..i + n]
                        .iter()
                        .flat_map(|x| x.related_information.iter().cloned())
                        .collect();
                    if !related.is_empty() {
                        crate::gostd::slices::sort_func(&mut related, compare_diagnostics);
                        related.dedup_by(|b, a| equal_diagnostics(a, b));
                        merged.set_related_info(related);
                    }
                }
                result.push(merged);
                i += n;
            }
            result
        }
        let parse = |name: &str| {
            let opts = SourceFileParseOptions {
                file_name: name.to_string(),
                ..Default::default()
            };
            parse_source_file(&opts, "let a = 1;\n", ScriptKind::TS).root
        };
        // "/B.ts" < "/a.ts" < "/a.ts" (another parse) < "/b.ts" < "/\u{e4}.ts" by bytes.
        let files = [
            Node::NIL,
            parse("/b.ts"),
            parse("/a.ts"),
            parse("/\u{e4}.ts"),
            parse("/B.ts"),
            parse("/a.ts"),
        ];
        let messages = [
            diag::Cannot_find_name_0,
            diag::Cannot_redeclare_block_scoped_variable_0,
            diag::Unused_ts_expect_error_directive,
        ];
        let mut seed = 0x2545_f491_4f6c_dd1d_u64;
        let mut next = |bound: u64| {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed % bound
        };
        let mut make = |next: &mut dyn FnMut(u64) -> u64| {
            let file = files[next(files.len() as u64) as usize];
            let pos = next(4) as i32;
            let message = messages[next(messages.len() as u64) as usize];
            let args = vec![["x", "y"][next(2) as usize].to_string()];
            new_diagnostic(
                file,
                TextRange::new(pos, pos + next(2) as i32),
                message,
                args,
            )
        };
        let mut diagnostics = Vec::new();
        for _ in 0..3000 {
            let mut d = make(&mut next);
            for _ in 0..next(3).saturating_sub(1) {
                let related = make(&mut next);
                d.add_related_info(Some(related));
            }
            diagnostics.push(d);
        }
        let expected = reference(diagnostics.clone());
        let got = sort_and_deduplicate_diagnostics(diagnostics);
        assert!(expected.len() < 1000, "the list has duplicates to merge");
        assert!(
            expected.iter().any(|d| d.related_information.len() > 2),
            "some merges join related information"
        );
        let text = |list: &[Diagnostic]| list.iter().map(|d| format!("{d:?}")).collect::<Vec<_>>();
        assert_eq!(text(&got), text(&expected));
        assert!(sort_and_deduplicate_diagnostics(Vec::new()).is_empty());
    }

    // Options that differ only in the `paths` order are different values:
    // each program keeps its own order.
    #[test]
    fn interned_options_keep_the_paths_order() {
        let with_paths = |keys: [&str; 2]| CompilerOptions {
            config_file_path: "/interned_options_keep_the_paths_order/tsconfig.json".into(),
            paths: Some(
                keys.iter()
                    .map(|&key| (key.to_string(), Some(vec![format!("./{key}")])))
                    .collect(),
            ),
            ..CompilerOptions::default()
        };
        let a = with_paths(["a/*", "*"]);
        let b = with_paths(["*", "a/*"]);
        assert_eq!(a, b, "the derived == ignores the order");
        assert!(!a.deep_equal(&b));
        assert!(a.deep_equal(&a.clone()));

        let interned_a = intern_compiler_options(&a);
        let interned_b = intern_compiler_options(&b);
        assert!(!std::ptr::eq(interned_a, interned_b));
        let keys = |options: &CompilerOptions| {
            options
                .paths
                .as_ref()
                .unwrap()
                .keys()
                .cloned()
                .collect::<Vec<_>>()
        };
        assert_eq!(keys(interned_a), ["a/*", "*"]);
        assert_eq!(keys(interned_b), ["*", "a/*"]);
        let again = intern_compiler_options(&b.clone());
        assert!(std::ptr::eq(again, interned_b));

        // The watch mode config check (Go `reflect.DeepEqual`) sees it too.
        use crate::frontend::tsoptions::parsed_options::ParsedOptions;
        let parsed = |options: CompilerOptions| ParsedOptions {
            compiler_options: std::rc::Rc::new(options),
            ..ParsedOptions::default()
        };
        assert!(parsed(a.clone()) != parsed(b));
        assert!(parsed(a.clone()) == parsed(a));
    }

    // Go: compiler/emitter.go:504-521. No Go test covers this part. The
    // expected values come from Go at pin B (16c25522e123): the same calls in
    // a Go test, and for the cases marked "oracle" also a `noEmit`
    // incremental tsgo run, where a JSON file that Go may emit is in
    // `affectedFilesPendingEmit`.
    #[test]
    fn json_file_may_be_emitted_matches_go() {
        // (file, outDir, rootDir, config file path, may be emitted)
        let cases = [
            // No outDir.
            ("/proj/src/data.json", "", "/proj/src", "", false),
            // No rootDir and no config file: the common directory is unknown.
            ("/proj/src/data.json", "/proj/dist", "", "", true),
            // oracle: under rootDir.
            ("/proj/src/data.json", "/proj/dist", "/proj/src", "", true),
            // oracle: outside rootDir, the output path is the file itself.
            ("/proj/data.json", "/proj/dist", "/proj/src", "", false),
            // oracle: outDir is rootDir.
            ("/proj/src/data.json", "/proj/src", "/proj/src", "", true),
            // oracle: outDir is the config file directory.
            (
                "/proj/src/data.json",
                "/proj",
                "",
                "/proj/tsconfig.json",
                true,
            ),
            // oracle: the path starts with rootDir, but not at a separator.
            (
                "/proj/src-data/data.json",
                "/proj/dist",
                "/proj/src",
                "",
                true,
            ),
            // A root keeps its separator.
            ("/data.json", "/", "/", "", false),
        ];
        for (file, out_dir, root_dir, config_file_path, expected) in cases {
            let options = CompilerOptions {
                out_dir: out_dir.to_string(),
                root_dir: root_dir.to_string(),
                config_file_path: config_file_path.to_string(),
                ..Default::default()
            };
            assert_eq!(
                json_file_may_be_emitted(file, &options, "/proj", true),
                expected,
                "{file} outDir {out_dir:?} rootDir {root_dir:?} config {config_file_path:?}"
            );
        }
    }

    #[test]
    fn json_file_may_be_emitted_case_insensitive() {
        let options = CompilerOptions {
            out_dir: "/proj/dist".to_string(),
            root_dir: "/proj/SRC".to_string(),
            ..Default::default()
        };
        assert!(json_file_may_be_emitted(
            "/proj/src/data.json",
            &options,
            "/proj",
            false
        ));
        assert!(!json_file_may_be_emitted(
            "/proj/src/data.json",
            &options,
            "/proj",
            true
        ));
    }
}
