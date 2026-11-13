//! Resolve ahead: worker threads resolve again, in the same order, the
//! module names that the previous program load of a language server project
//! resolved, while the loader of the new load runs (lspp95 plan, step 1).
//! They also find again the package scope of each directory whose files the
//! previous load read (Go `loadSourceFileMetaData`), and the loader takes a
//! scope as it takes a module answer (`Caches::package_scope_ahead`).
//!
//! The loader stays serial. It takes a worker answer only when the answer
//! passes its host's check (`ResolveAheadHost::accept`). The worker logs
//! the file system calls of its resolution with their answers
//! (`AheadCall`); a call that the check cannot do again (a directory
//! listing, a stat, a read of an open file) makes the answer unshareable.
//! The check gives each call to the host's snapshot file system as the
//! loader's own call would go there, at the moment the loader's own
//! resolution would make it:
//!
//! - a read is the loader's read, and its text must have the worker's hash
//!   (a worker read that failed fails);
//! - a `file_exists`, `directory_exists` or `realpath` must agree with the
//!   snapshot's answer for the name, if it has one. The snapshot's lookup
//!   cache takes each answer that it does not have from the workers' lookups
//!   of the load (`WorkerStats`, `AheadLookups`) before it asks the OS, as
//!   Go's parse tasks fill one per-snapshot cache. So a worker's answer,
//!   which the worker got during this load, is the snapshot's answer when
//!   the snapshot had none. A `file_exists` answer that the workers know
//!   from an earlier load is asked of the snapshot's file system.
//!
//! So the load has one answer for each path, also when the disk changes
//! during the load. The check also replays the side effects of the calls
//! (seen files, missing directories). If an answer does not pass, the
//! loader resolves the key itself. So the program, its file order and the
//! watched files are those of a load with no workers whose calls found the
//! disk as the workers found it.
// PORT: not in Go (perf). Go resolves in each parse task, on all threads;
// the port's loader resolves on one thread (contract 10).

use crate::frontend::prelude::*;
use std::cell::Cell;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Condvar, Mutex};
use crate::xxhash_rust::xxh3::xxh3_128;

/// Whether program loads resolve ahead (`GOPORT_RESOLVE_AHEAD`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// `0`: no workers. The loader resolves every key itself.
    Off,
    /// The default: the workers resolve while the loader runs.
    On,
    /// `force`: the workers resolve every key before the loader starts, so
    /// the loader finds an answer for each key of the previous load (tests).
    Force,
}

thread_local! {
    /// `set_mode` on this thread.
    static MODE: Cell<Option<Mode>> = const { Cell::new(None) };
    /// The counts of the last resolve-ahead load on this thread.
    static LAST_STATS: Cell<Option<LoadStats>> = const { Cell::new(None) };
    /// `inject_worker_panic` on this thread.
    static INJECT_PANIC: Cell<bool> = const { Cell::new(false) };
}

/// Sets the mode of the program loads on this thread, in place of
/// `GOPORT_RESOLVE_AHEAD` (`None`: the variable again). For tests.
pub fn set_mode(mode: Option<Mode>) {
    MODE.with(|m| m.set(mode));
}

/// The mode of a program load on this thread.
#[must_use]
pub fn mode() -> Mode {
    static ENV: std::sync::OnceLock<Mode> = std::sync::OnceLock::new();
    MODE.with(Cell::get).unwrap_or_else(|| {
        *ENV.get_or_init(|| match std::env::var("GOPORT_RESOLVE_AHEAD").as_deref() {
            Ok("0") => Mode::Off,
            Ok("force") => Mode::Force,
            _ => Mode::On,
        })
    })
}

/// The counts of one resolve-ahead load.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LoadStats {
    /// Module keys of the previous load that the workers had.
    pub keys: usize,
    /// Module keys that this load resolved or took.
    pub new_keys: usize,
    /// Package scope keys of the previous load that the workers had
    /// (`KeyList::push_scope`).
    pub scopes: usize,
    /// Package scope keys of this load.
    pub new_scopes: usize,
    pub loader: AheadStats,
    /// The known files that the workers of this load started with
    /// (`WorkerState::known_files`). 0 when the load had no workers.
    pub known_files: usize,
    /// A worker panicked outside a resolution during this load, and the
    /// loader resolved the rest of the keys itself (`run_task`).
    pub worker_panic: bool,
}

/// The counts of the last resolve-ahead load on this thread. For tests.
#[must_use]
pub fn last_stats() -> Option<LoadStats> {
    LAST_STATS.with(Cell::get)
}

/// Makes the workers of the loads on this thread panic outside a
/// resolution, as a port bug would (`run_task`), after they resolved their
/// keys: the answers are published, and the load must take none of them
/// (`AheadQueue::fail`). For tests.
#[doc(hidden)]
pub fn inject_worker_panic(on: bool) {
    INJECT_PANIC.with(|inject| inject.set(on));
}


/// The guard of `hold_workers`.
#[doc(hidden)]
pub struct HeldWorkers(());





/// The workers' view of a host's file system: the OS file system with the
/// open files of the host over it, as the language server's overlay file
/// system shows it (project/overlayfs.rs).
pub struct WorkerView {
    /// `current_directory` and `use_case_sensitive_file_names` make the
    /// paths, as the host's `to_path`.
    pub current_directory: String,
    pub use_case_sensitive_file_names: bool,
    /// The paths of the open files.
    pub open_files: FxHashSet<Path>,
    /// The paths of the directories that have open files in them.
    pub open_directories: FxHashSet<Path>,
}


/// A project's share in what the workers keep from job to job: the known
/// files and the package.json parses. Each host of the project's loads
/// gives it to the next one (project/compilerhost.rs). A project that a
/// snapshot clone made and deleted gives it to the session's stash, and the
/// project of the same config that a later clone makes takes it
/// (`ResolveAheadStash`). When the last one drops it (the project's
/// programs are released, or the stash drops the project) after a load of
/// the project gave the workers a job, the workers drop what they keep
/// (`Workers::forget`). So the kept state holds only what the loads of
/// live and stashed projects found, and the next loads find it again.
// PORT: not in Go (perf).
#[derive(Default)]
pub struct KeptShare {
    /// The epoch of the kept state (`WorkerState::epoch`) when a load of
    /// the project last gave the workers a job.
    epoch: Cell<Option<u64>>,
}





























const STAT_SHARDS: usize = 16;


/// The lookup answers of a resolve-ahead job (`WorkerStats`), which the
/// workers got during the job's load. The host's snapshot file system takes
/// an answer from them before it asks the OS
/// (`ResolveAheadHost::attach`).
pub trait AheadLookups: Send + Sync {
    fn file_exists(&self, name: &str) -> Option<bool>;
    fn directory_exists(&self, name: &str) -> Option<bool>;
    fn realpath(&self, name: &str) -> Option<String>;
}










fn sorted(paths: &FxHashSet<Path>) -> Vec<&str> {
    let mut paths: Vec<&str> = paths.iter().map(Path::as_str).collect();
    paths.sort_unstable();
    paths
}


#[cfg(test)]
mod tests {
    use super::{
        AheadLookups, AheadQueue, Job, KeyList, ResolverConfig, Task, WorkerStats, WorkerView,
        Workers, run_task, write_debug_line,
    };
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    /// A pool with no threads, for `run_task` on the test thread.
    fn test_workers() -> Workers {
        Workers {
            state: std::sync::Mutex::default(),
            wake: std::sync::Condvar::new(),
            left: std::sync::Condvar::new(),
            count: AtomicUsize::new(1),
        }
    }

    /// A job of `keys` in a missing directory.
    fn test_job(keys: KeyList, inject_panic: bool) -> Arc<Job> {
        let root = "/goport-resolve-ahead-test-missing";
        Arc::new(Job {
            queue: Arc::new(AheadQueue::new(Arc::new(keys))),
            epoch: 0,
            known_files: Arc::default(),
            closed: AtomicBool::new(false),
            left: AtomicUsize::new(0),
            answers: Arc::default(),
            view: WorkerView {
                current_directory: root.to_string(),
                use_case_sensitive_file_names: true,
                open_files: Default::default(),
                open_directories: Default::default(),
            },
            stats: Arc::default(),
            config: Arc::new(ResolverConfig {
                options: crate::options::CompilerOptions::default(),
                typings_location: String::new(),
                project_name: String::new(),
                extra_extensions: Vec::new(),
                current_directory: root.to_string(),
            }),
            inject_panic,
        })
    }

    // PORT: not in Go (resolve ahead). R169 reviewer (followups19): a worker
    // panic fails the job but does not close it. The loader can still take
    // an answer that another worker published before it saw the failure,
    // and the check passes the lookups that the snapshot takes from the job
    // (`ResolveAheadHost::attach`), so those lookups must be stored. When
    // the panic closed the job, they were not, and the snapshot asked the
    // OS for them again later in the load.
    #[test]
    fn a_worker_panic_fails_the_job_but_keeps_its_lookups() {
        let workers = test_workers();
        let job = test_job(KeyList::default(), true);
        run_task(&workers, Task::Run(job.clone()));
        assert!(job.queue.failed());
        assert_eq!(job.left.load(Ordering::Relaxed), 1);
        assert!(!job.closed.load(Ordering::Relaxed));
        // Another worker's lookup during the load.
        assert!(job.stats.file_exists(&job.closed, "/p/a.ts", || true));
        assert_eq!(
            AheadLookups::file_exists(&*job.stats, "/p/a.ts"),
            Some(true)
        );
    }

    // PORT: not in Go (resolve ahead). The workers stop at a failed job
    // (`AheadQueue::fail`) without a close: the loader resolves its keys.
    #[test]
    fn a_worker_takes_no_key_of_a_failed_job() {
        let workers = test_workers();
        let mut keys = KeyList::default();
        keys.push("/p", "./a", crate::options::RESOLUTION_MODE_NONE);
        let job = test_job(keys, false);
        job.queue.fail();
        run_task(&workers, Task::Run(job.clone()));
        assert_eq!(job.left.load(Ordering::Relaxed), 1);
        assert_eq!(job.queue.next.load(Ordering::Relaxed), 0);
    }

    // PORT: not in Go (resolve ahead). R167 reviewer (followups15a): a
    // worker of an ended job must not store a lookup, since the snapshot
    // keeps the job's lookups as answers of the load (`Workers::end`). A
    // lookup that another worker stored before the end still answers.
    #[test]
    fn an_ended_job_stores_no_lookup() {
        let stats = WorkerStats::default();
        let closed = AtomicBool::new(false);
        assert!(stats.file_exists(&closed, "/p/a.ts", || true));
        assert!(stats.directory_exists(&closed, "/p", || (true, None)).0);
        closed.store(true, Ordering::Relaxed);
        stats.wait_for_stores();
        assert!(stats.file_exists(&closed, "/p/a.ts", || false));
        assert!(!stats.file_exists(&closed, "/p/b.ts", || false));
        assert!(!stats.directory_exists(&closed, "/q", || (false, None)).0);
        let real = stats.realpath(&closed, "/p/l", || {
            let call = super::AheadCall::Realpath {
                name: "/p/l".to_string(),
                real: "/p/r".to_string(),
            };
            ("/p/r".to_string(), std::sync::Arc::new(call))
        });
        assert_eq!(real.0, "/p/r");
        assert_eq!(AheadLookups::file_exists(&stats, "/p/a.ts"), Some(true));
        assert_eq!(AheadLookups::directory_exists(&stats, "/p"), Some(true));
        assert_eq!(AheadLookups::file_exists(&stats, "/p/b.ts"), None);
        assert_eq!(AheadLookups::directory_exists(&stats, "/q"), None);
        assert_eq!(AheadLookups::realpath(&stats, "/p/l"), None);
    }

    // PORT: not in Go (the resolve-ahead debug log). The workers' panic
    // lines and the loader's counts go to one file at the same time
    // (followups13b skeptic: 1,917 of 12,288 lines garbled).
    #[test]
    fn debug_log_lines_of_parallel_writers_do_not_interleave() {
        let to = std::env::temp_dir().join(format!(
            "goport-resolve-ahead-log-{}.txt",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&to);
        let to_text = to.to_str().expect("a UTF-8 temp path").to_string();
        std::thread::scope(|scope| {
            for thread in 0..8 {
                let to_text = &to_text;
                scope.spawn(move || {
                    for line in 0..500 {
                        write_debug_line(
                            to_text,
                            format_args!(
                                "resolve-ahead: a worker panicked in a {} task: {} {}",
                                "job", thread, line
                            ),
                        );
                    }
                });
            }
        });
        let text = std::fs::read_to_string(&to).expect("the log");
        std::fs::remove_file(&to).expect("remove the log");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 8 * 500);
        let bad = lines
            .iter()
            .filter(|line| {
                let Some(rest) =
                    line.strip_prefix("resolve-ahead: a worker panicked in a job task: ")
                else {
                    return true;
                };
                let mut parts = rest.split(' ');
                !(parts
                    .next()
                    .is_some_and(|t| t.parse::<u32>().is_ok_and(|t| t < 8))
                    && parts
                        .next()
                        .is_some_and(|l| l.parse::<u32>().is_ok_and(|l| l < 500))
                    && parts.next().is_none())
            })
            .count();
        assert_eq!(bad, 0, "{bad} lines garbled");
    }
}
