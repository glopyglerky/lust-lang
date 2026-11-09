//! PORT: not in Go (perf). Go parses the configs of a build in parallel:
//! `createBuildTasks` queues the parse of each config and of its
//! references on a work group (orchestrator.go:174). Here the configs parse
//! on the orchestrator thread, because `ParsedCommandLine` is not `Send`.
//! Most of a parse is the match of the `include` specs against the file
//! system (Go `getFileNamesFromConfigSpecs`). So threads parse the configs
//! of the build ahead of the orchestrator, each on the OS file system of
//! its thread with its own caches, and keep the file names that each
//! config's specs match. The orchestrator's parse of a config takes them
//! (`BuildHost`'s `ParseConfigHost::get_file_names_from_config_specs`)
//! when it matches the same specs from the same base path with the same
//! extensions, and waits for a thread that is still matching them. The
//! file names are a function of these inputs and of the file system, and
//! the build writes nothing before its graph is made. A config that no
//! thread has started yet is the orchestrator's own: it matches the specs
//! itself and queues the references it finds.
//!
//! Go caches the lookups of each match (`GetAccessibleEntries`,
//! `Realpath`) in the build host's `cachedvfs` for the rest of the build,
//! and a later program reads them: for example the listing of a typeRoots
//! directory for its automatic type directives, after the build wrote
//! into that directory. So a thread records the lookups of its match, and
//! when the orchestrator takes the match, they go into the host's cache
//! (`BuildStatCache::add`), as if the orchestrator had made them. The
//! thread made them before the build wrote anything, as Go does.
//!
//! The same threads read the build info file of each task ahead of its
//! up-to-date check (`BuildInfoRead`; orchestrator.rs `BuildInfoPrefetch`).
//! Go reads it in the check, on the builder goroutine of the task
//! (buildtask.go `loadOrStoreBuildInfo`). A thread queues the read of the
//! build info file of a config when it has parsed it, and the orchestrator
//! queues the read of a config that it parsed (`queue_read`), so the reads
//! run while the graph is made. The threads take the queued configs first:
//! the orchestrator waits for each config before it makes the graph. When
//! the graph is made, the orchestrator keeps the reads that its checks can
//! use (`finish_reads`; the rules are in orchestrator.rs
//! `start_build_info_prefetch`) and drops the others. A read only reads,
//! and the build writes nothing before its graph is made. For those rules,
//! a thread also finds the key of the build info file of each config it
//! parsed (`PathKeys::build_info_key`) before the orchestrator can take the
//! config, so before the graph is made, as the orchestrator does for the
//! other keys (`build_info_key`).
//!
//! The pool starts a thread for each queued job that no thread is free
//! for, up to `MAX_PREFETCH_THREADS` and the cores. A new thread starts the
//! other new threads (`spawn_threads`), so the thread that queues pays for
//! one thread start.

use crate::execute::build::build_task::{StatusCheckOptions, };
use crate::execute::build::shared_outputs::PathKeys;
use crate::frontend::prelude::*;
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};

/// The most threads of a `PrefetchPool`. The threads only read, so their
/// count changes no output; it is not Go's `numRoutines`. On 8 cores with
/// 16 hardware threads, 12 or 16 threads made the noop of 48 small
/// projects 1 ms slower than 8 (noop1, mini-abf9): the orchestrator thread
/// shares a core with a prefetch thread.
const MAX_PREFETCH_THREADS: usize = 8;

/// What `get_file_names_from_config_specs` reads, other than the file
/// system: two matches with equal inputs give the same file names.
#[derive(Clone, PartialEq, Eq)]
struct MatchInputs {
    base_path: String,
    files: Vec<String>,
    include: Vec<String>,
    exclude: Vec<String>,
    /// `get_supported_extensions` and its `WithJsonIfResolveJsonModule`
    /// form: the only parts of the options that the match reads.
    extensions: Vec<Vec<String>>,
    extensions_with_json: Vec<Vec<String>>,
}

impl MatchInputs {
}

/// The file names that a thread matched for one config.
struct MatchedFileNames {
    inputs: MatchInputs,
    file_names: Vec<String>,
    literal_file_names_len: i32,
    /// The cached lookups that the match made (`RecordingFs`).
    lookups: StatCache,
}

enum SlotState {
    /// No thread has started the config.
    Queued,
    /// A thread parses the config.
    Running,
    /// The thread's match, `None` when the parse matched nothing (no
    /// config file, no options) or panicked.
    Done(Option<MatchedFileNames>),
    /// The thread's work after the parse (`parse_config`) panicked with
    /// this payload. The orchestrator panics with it when it takes the
    /// slot.
    Panicked(Box<dyn std::any::Any + Send>),
    /// The orchestrator took the slot.
    Taken,
}

struct Slot {
    state: Mutex<SlotState>,
    done: Condvar,
}

/// One build info file that a thread reads ahead of the up-to-date check
/// of its task: the file, the root files of the task (`resolved.FileNames()`)
/// and the options that the check reads.
#[derive(PartialEq)]
pub(crate) struct BuildInfoRead {
    pub(crate) name: String,
    pub(crate) input_files: Vec<String>,
    pub(crate) check: StatusCheckOptions,
}

impl BuildInfoRead {
}






















/// The parse config host of a pool thread: records the match of the
/// config it parses.
struct ThreadConfigHost {
    fs: Rc<dyn Fs>,
    current_directory: String,
    /// The config name and match of the last `get_file_names_from_config_specs`.
    matched: RefCell<Option<(String, MatchedFileNames)>>,
}

/// The file system of a pool thread's match: `fs`, with each cached
/// lookup (Go `cachedvfs`: all but `Stat`) recorded in `lookups`.
struct RecordingFs<'a> {
    fs: &'a dyn Fs,
    lookups: StatCache,
}


#[cfg(test)]
mod tests {
    use super::*;

    /// The configs whose work after the parse panics (`panic_after_parse`).
    static PANIC_AFTER_PARSE: Mutex<Vec<String>> = Mutex::new(Vec::new());

    /// Test-only panic hook of `parse_config`: panics after the parse of a
    /// config in `PANIC_AFTER_PARSE`, where the key and queue calls can
    /// panic.
    pub(super) fn panic_after_parse(config: &str) {
        if lock(&PANIC_AFTER_PARSE).iter().any(|name| name == config) {
            panic!("test panic after the parse of {config}");
        }
    }

    // A panic in a thread's work after the parse (build info key, queued
    // references) reaches the orchestrator's take as a panic. Before, the
    // slot stayed Running and `tsc -b` waited for it forever.
    #[test]
    fn panic_after_parse_reaches_take() {
        let dir =
            std::env::temp_dir().join(format!("ts_goport_prefetch_panic_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.ts"), "export {};").unwrap();
        std::fs::write(dir.join("tsconfig.json"), r#"{"files": ["a.ts"]}"#).unwrap();
        // The compiler takes normalized paths: a Windows temp dir has backslashes.
        let normalized =
            |p: &std::path::Path| crate::frontend::tspath::normalize_slashes(p.to_str().unwrap());
        let config = normalized(&dir.join("tsconfig.json"));
        lock(&PANIC_AFTER_PARSE).push(config.clone());
        let compare_paths_options = ComparePathsOptions {
            use_case_sensitive_file_names: true,
            current_directory: normalized(&dir),
        };
        let path = to_path(&config, &compare_paths_options.current_directory, true);
        let pool = PrefetchPool::start(
            CompilerOptions::default(),
            None,
            &compare_paths_options,
            false,
            std::slice::from_ref(&config),
        )
        .expect("a prefetch thread starts");
        // The orchestrator parses a config that no thread has started, so
        // the take waits until a thread has started it.
        let slot = lock(&pool.shared.slots).get(&path).cloned().unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
        while matches!(*lock(&slot.state), SlotState::Queued) {
            assert!(
                std::time::Instant::now() < deadline,
                "no thread started the config"
            );
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        let inputs = MatchInputs {
            base_path: String::new(),
            files: Vec::new(),
            include: Vec::new(),
            exclude: Vec::new(),
            extensions: Vec::new(),
            extensions_with_json: Vec::new(),
        };
        // A take that waits forever fails the test after the timeout.
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let taken = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                pool.take(&path, &inputs).is_some()
            }));
            let _ = sender.send(taken.map_err(|payload| {
                payload
                    .downcast_ref::<String>()
                    .cloned()
                    .unwrap_or_default()
            }));
        });
        let taken = receiver.recv_timeout(std::time::Duration::from_secs(60));
        lock(&PANIC_AFTER_PARSE).retain(|name| *name != config);
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(
            taken,
            Ok(Err(format!("test panic after the parse of {config}")))
        );
    }

    // A pool thread that a panic outside the catch_unwind of a job ends
    // (the config host of `run_thread`) counts as ended, and its job as
    // done. Before, it stayed counted as a busy thread, and a pool whose
    // threads all ended so started no thread for the build info reads,
    // whose checks then waited forever.
    #[test]
    fn thread_ended_by_a_panic_is_not_counted() {
        let pool = PrefetchPool::new(None, &ComparePathsOptions::default(), false);
        {
            let mut queue = lock(&pool.shared.queue);
            queue.threads = 1;
            queue.busy = 1;
        }
        let shared = pool.shared.clone();
        let ended = std::thread::spawn(move || {
            let _thread = ThreadEnd {
                shared: &shared,
                busy: true,
            };
            panic!("test panic in the config host");
        })
        .join();
        assert!(ended.is_err());
        let queue = lock(&pool.shared.queue);
        assert_eq!((queue.threads, queue.busy), (0, 0));
    }
}
