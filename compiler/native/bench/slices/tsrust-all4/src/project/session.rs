//! Go `internal/project/session.go`.
//!
//! PORT: one thread (see `project/dirty/interfaces.rs`). Every Go mutex of
//! `Session` is dropped; fields that Go changes after construction are
//! `Cell` / `RefCell`. Go `*Session` is `Rc<Session>`: methods that queue
//! work capturing the session (background tasks, timers) or that hand the
//! session to snapshot code take `self: &Rc<Self>`.
//!
//! Background tasks (`backgroundQueue.Enqueue`) run through
//! `background::Queue`, which posts them to `gostd::local::go` in Go
//! enqueue order. Debounce sleeps, the idle cache clean timer and the
//! telemetry ticker are `gostd::local::after_func` timers, so their
//! functions run on the dispatch thread; a debounced task keeps a
//! `background::TaskHold` until its timer has run. `WaitForBackgroundTasks`
//! drains `gostd::local` through `Queue::wait`. The one exception is the
//! clone of the auto-import warm, which is `gostd::local` idle work: the
//! LSP server runs it only when no message waits, and the reader thread can
//! cancel it or make it yield (`WarmAutoImportPreempt`).
//!
//! Go runtime metrics (`runtime/metrics`) exist only in the Go runtime.
//! Performance telemetry reads them as `KindBad` (`metrics_read`), so its
//! Go runtime fields are 0; the system memory fields come from
//! `/proc/meminfo` (`osmemory_get`). The log-only runtime metrics and
//! `runtime.GC()` are PORT skips.

use crate::project::prelude::*;

use crate::contentmapper;
use std::cell::Cell;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

// Go: project/session.go:36 UpdateReason
// PORT: Go `type UpdateReason int` with iota consts. Go
// `UpdateReasonDidOpenFile` is `UpdateReason::DID_OPEN_FILE` (same values).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UpdateReason(pub i32);

impl UpdateReason {
    pub const UNKNOWN: UpdateReason = UpdateReason(0);
    pub const DID_OPEN_FILE: UpdateReason = UpdateReason(1);
    pub const DID_CLOSE_FILE: UpdateReason = UpdateReason(2);
    pub const DID_CHANGE_COMPILER_OPTIONS_FOR_INFERRED_PROJECTS: UpdateReason = UpdateReason(3);
    pub const REQUESTED_LANGUAGE_SERVICE_PENDING_CHANGES: UpdateReason = UpdateReason(4);
    pub const REQUESTED_LANGUAGE_SERVICE_PROJECT_NOT_LOADED: UpdateReason = UpdateReason(5);
    pub const REQUESTED_LANGUAGE_SERVICE_FOR_FILE_NOT_OPEN: UpdateReason = UpdateReason(6);
    pub const REQUESTED_LANGUAGE_SERVICE_PROJECT_DIRTY: UpdateReason = UpdateReason(7);
    pub const REQUESTED_LOAD_PROJECT_TREE: UpdateReason = UpdateReason(8);
    pub const REQUESTED_LANGUAGE_SERVICE_WITH_AUTO_IMPORTS: UpdateReason = UpdateReason(9);
    pub const IDLE_CLEAN_DISK_CACHE: UpdateReason = UpdateReason(10);
    pub const DID_CHANGE_CONFIG_FILE: UpdateReason = UpdateReason(11);
    // tsgo#4712
    pub const DID_CHANGE_CONTENT_MAPPER_CONTRIBUTIONS: UpdateReason = UpdateReason(12);
}

// Go: project/session.go:39 ErrNoProjectForUnknownScriptKind (tsgo#4712)
// ErrNoProjectForUnknownScriptKind identifies requests for otherwise unsupported files.

// Go: project/session.go:57 ContentMapperContributions (tsgo#4712)
// PORT: Go `[]*contentmapper.Mapper` is `Vec<Rc<Mapper>>`.
#[derive(Clone, Debug, Default)]
pub struct ContentMapperContributions {
    pub mappers: Vec<Rc<contentmapper::Mapper>>,
    pub extensions: Vec<String>,
}

// Go: project/session.go:64 watchRequestTimeout
// watchRequestTimeout is the maximum time to wait for the client to respond to
// a WatchFiles or UnwatchFiles request while holding the watches mutex.
pub const WATCH_REQUEST_TIMEOUT: Duration = Duration::from_secs(1);

// Go: project/session.go:68 SessionOptions
// SessionOptions are the immutable initialization options for a session.
// Snapshots may reference them as a pointer since they never change.
// PORT: Go `*SessionOptions` is `Rc<SessionOptions>`.
pub struct SessionOptions {
    pub current_directory: String,
    pub default_library_path: String,
    pub typings_location: String,
    pub position_encoding: lsproto::PositionEncodingKind,
    pub watch_enabled: bool,
    pub logging_enabled: bool,
    pub telemetry_enabled: bool,
    pub push_diagnostics_enabled: bool,
    // RunExternalCode allows configured content mappers to run their (external) processes,
    // gated on workspace trust by the client. It corresponds to the --runExternalCode CLI flag.
    pub run_external_code: bool,
    pub debounce_delay: Duration,
    pub checker_pool_options: CheckerPoolOptions,
}

// Go: project/session.go:84 SessionInit
// PORT: Go nil interfaces and pointers are `None`.

// Go: project/session.go:104 Session
// Session manages the state of an LSP session. It receives textDocument
// events and requests for LanguageService objects from the LPS server
// and processes them into immutable snapshots as the data source for
// LanguageServices. When Session transitions from one snapshot to the
// next, it diffs them and updates file watchers and Automatic Type
// Acquisition (ATA) state accordingly.
// PORT: the Go mutexes (`snapshotMu`, `snapshotUpdateMu`,
// `scheduledSnapshotUpdateMu`, `userConfigRWMu`, `pendingFileChangesMu`,
// `pendingATAChangesMu`, `diagnosticsRefreshMu`, `warmAutoImportMu`,
// `idleCacheCleanMu`) are dropped. Go `atomic.*` fields are `Cell`.
// PORT: Go embeds `*SnapshotHost` (ts#64163); here it is the field
// `snapshot_host`, and `Session` derefs to it, so the host's fields and
// methods are promoted as in Go. A `Session` method with the same name
// (`fs`, `get_current_directory`, `close`) wins, as in Go.

// Go: project/session.go:199 newContentMapperHost (tsgo#4712)
// newContentMapperHost creates the session's shared content mapper host when the workspace is trusted and
// a spawner is available; otherwise it returns nil, and configured content mappers are rejected by the
// config-file gate.

// Go: project/session.go:210 NewSession

// PORT: Go `Session` embeds `*SnapshotHost` (ts#64163).

// PORT: Go `FS()` and `GetCurrentDirectory()` implement
// `module.ResolutionHost`, whose Rust form returns borrowed values. The
// inherent methods keep the Go results for other callers (api session).

// Go: project/session.go:1827 NpmInstall
// PORT: Go `NpmInstall` implements `ata.NpmExecutor`. With this impl and
// `module::ResolutionHost`, `Session` is an `ata::TypingsInstallerHost`.



/// PORT: the longest time that a message waits for an eager attempt of the
/// auto-import warm (see `run_pending_warm`). Go's head start is about 2 ms
/// in the lswarm1 repros.
pub const WARM_AUTO_IMPORT_HOLD_CAP: Duration = Duration::from_millis(5);

/// PORT: how a clone attempt of the auto-import warm ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WarmAttemptEnd {
    /// The clone ended. Go adopts it.
    Done,
    /// The warm was cancelled (Go `warmCtx`). Go discards the clone.
    Cancelled,
    /// Only the attempt was cancelled: a message did not wait for it. The
    /// warm tries again.
    Yielded,
}

/// PORT: lets the LSP reader thread cancel the auto-import warm, or make it
/// wait or yield.
///
/// Go runs the warm on a goroutine, and the dispatch goroutine cancels it
/// when it handles didOpen, didChange, didClose or didChangeWatchedFiles.
/// Other messages run at the same time as the warm. Here the clone runs on
/// the dispatch thread (`Session::run_pending_warm`), so the dispatch thread
/// cannot get to a message until the clone ends. The reader thread calls
/// `on_message` for each message that it queues:
///
/// - While an eager attempt runs, the message first waits up to the
///   attempt's hold for the clone to end (Go's head start).
/// - Then a file event cancels the warm, and the warm stops at its next
///   context check. The handler's own `cancel_warm_auto_import_cache` then
///   finds the warm done, as Go's does when the warm has ended.
/// - Any other message yields an eager attempt: the clone stops at its next
///   context check and the warm tries again later. During an attempt that
///   is not eager, such a message waits for the whole clone.
///
/// It holds the same context and cancel function as
/// `Session::warm_auto_import_cancel`, and is set and cleared with it. One
/// mutex covers the warm, its attempt and the queueing of a message, so an
/// attempt does not start while a message waits (`set_busy`), and after an
/// attempt ends the reader neither cancels nor yields it.
#[derive(Clone, Default)]
pub struct WarmAutoImportPreempt(Arc<WarmAutoImportPreemptShared>);

#[derive(Default)]
struct WarmAutoImportPreemptShared {
    entry: Mutex<Option<WarmAutoImportPreemptEntry>>,
    /// Signalled when an attempt ends.
    attempt_ended: Condvar,
    /// Whether a message waits for the dispatch thread (`set_busy`).
    busy: OnceLock<Box<dyn Fn() -> bool + Send + Sync>>,
}

struct WarmAutoImportPreemptEntry {
    ctx: Context,
    cancel: gostd::context::CancelFunc,
    file_name: String,
    /// The clone attempt that runs now: its cancel function and its hold
    /// (`None` when it is not eager).
    attempt: Option<(gostd::context::CancelFunc, Option<Duration>)>,
    /// Whether the clone ended. Go's warm has then ended, and its cancel
    /// does nothing.
    done: bool,
}

impl WarmAutoImportPreempt {
    fn entry(&self) -> MutexGuard<'_, Option<WarmAutoImportPreemptEntry>> {
        // PORT: Go mutexes do not poison.
        self.0.entry.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn set(&self, ctx: Context, cancel: gostd::context::CancelFunc, file_name: String) {
        *self.entry() = Some(WarmAutoImportPreemptEntry {
            ctx,
            cancel,
            file_name,
            attempt: None,
            done: false,
        });
    }

    fn clear(&self) {
        *self.entry() = None;
    }

    /// Sets the check that a message waits for the dispatch thread (the LSP
    /// server's request queue). An attempt does not start while it is true.
    pub fn set_busy(&self, busy: Box<dyn Fn() -> bool + Send + Sync>) {
        let _ = self.0.busy.set(busy);
    }

    /// Whether a message waits for the dispatch thread (`set_busy`).
    fn busy(&self) -> bool {
        self.0.busy.get().is_some_and(|busy| busy())
    }


    fn cancel_entry(entry: &mut Option<WarmAutoImportPreemptEntry>, logger: &dyn logging::Logger) {
        let Some(entry) = entry.take() else {
            return;
        };
        if entry.done || entry.ctx.err().is_some() {
            return;
        }
        logger.logf(&format!(
            "Cancelling auto-import warming for file {}",
            entry.file_name
        ));
        (entry.cancel)();
    }


    /// Starts a clone attempt, unless a message waits.
    fn start_attempt(&self, cancel: gostd::context::CancelFunc, hold: Option<Duration>) -> bool {
        let mut entry = self.entry();
        if self.busy() {
            return false;
        }
        if let Some(entry) = entry.as_mut() {
            entry.attempt = Some((cancel, hold));
        }
        true
    }

    /// Ends the clone attempt that runs, and says how it ended.
    fn end_attempt(&self, warm_ctx: &Context, attempt_ctx: &Context) -> WarmAttemptEnd {
        let mut entry = self.entry();
        let end = if warm_ctx.err().is_some() {
            WarmAttemptEnd::Cancelled
        } else if attempt_ctx.err().is_some() {
            WarmAttemptEnd::Yielded
        } else {
            WarmAttemptEnd::Done
        };
        if let Some(entry) = entry.as_mut() {
            entry.attempt = None;
            entry.done = end == WarmAttemptEnd::Done;
        }
        self.0.attempt_ended.notify_all();
        end
    }
}

// Go: project/session.go:661 idleCacheCleanDelay
pub const IDLE_CACHE_CLEAN_DELAY: Duration = Duration::from_secs(30);


// Go: project/session.go:703 performanceTelemetryInterval
pub const PERFORMANCE_TELEMETRY_INTERVAL: Duration = Duration::from_secs(5 * 60);

// PORT: Go `runtime/metrics.Sample` and `metrics.Value` (Go standard
// library). Only the shape is ported: the values describe the Go runtime.
#[derive(Clone, Debug, Default)]
struct MetricsSample {
    name: &'static str,
    value: MetricsValue,
}

// PORT: Go `metrics.Value` by `Kind()`.
#[derive(Clone, Copy, Debug, Default)]
enum MetricsValue {
    #[default]
    Bad,
    Uint64(u64),
    Float64(f64),
    Float64Histogram,
}

// Go: runtime/metrics/sample.go:45 Read (go1.27.1), which runs
// runtime/metrics.go:1029 readMetricsLocked. go1.27.1 returns early for no
// samples; the loop below does nothing then too.
// PORT: Go computes each metric from Go runtime statistics (heap, GC,
// scheduler, goroutines). The port has no Go runtime, so it has only
// `/sched/gomaxprocs:threads` (`gostd::runtime::gomaxprocs`, as Go
// runtime/metrics.go `gomaxprocs`). Go gives `KindBad` for a name it does
// not have, so every other sample is `KindBad` and the other Go runtime
// fields of the telemetry event are 0 (`memoryUsedBytes`, `goMemLimit`,
// `goGCPercent`, the heap and GC fields, `goroutineCount`, `gcCpuSeconds`,
// `userCpuSeconds`).

// PORT: Go `go-osstat/memory.Stats` (the two fields session.go reads).
struct OsMemoryStats {
    total: u64,
    used: u64,
}

// Go: github.com/mackerelio/go-osstat@v0.2.7 memory/memory_linux.go:15 Get
// Get memory statistics

// Go: github.com/mackerelio/go-osstat@v0.2.7 memory/memory_linux.go:33 collectMemoryStats
// PORT: Go fills every `Stats` field. The port keeps the fields that
// `Total` and `Used` need (`OsMemoryStats`); the other `memStats` names set
// only fields that nobody reads. Go reads lines with `bufio.Scanner`, which
// stops at the first read error; the port reads the whole file first. A
// meminfo line is never longer than the 64 KiB scanner limit.


// Go: project/session.go:976 setTristate

// Go: project/session.go:984 boolTelemetry
pub fn bool_telemetry(v: bool) -> String {
    if v {
        return "true".to_string();
    }
    "false".to_string()
}

// Go: project/session.go:991 countFileStats
// PORT: Go returns `*lsproto.ProjectInfoTelemetryMeasurements`; the event
// field holds the value.


// Go: project/session.go:1461 hasContentMapperOperationTimings (ts#64015)
// PORT: Go map order is random; the result does not depend on it.
pub fn has_content_mapper_operation_timings(
    timings: &IndexMap<String, contentmapper::MapperTimings>,
) -> bool {
    for timing in timings.values() {
        if has_content_mapper_operation_timing(timing) {
            return true;
        }
    }
    false
}

// Go: project/session.go:1470 hasContentMapperOperationTiming (ts#64015)
pub fn has_content_mapper_operation_timing(timing: &contentmapper::MapperTimings) -> bool {
    timing.spawn.count != 0
        || timing.open_project.count != 0
        || timing.close_project.count != 0
        || timing.transform.count != 0
}

// Go: project/session.go:1481 updateWatch
// PORT: Go `*WatchedFiles[T]` arguments are `Option<&WatchedFiles<T>>`.
// Go `logger != nil` compares the interface, which always holds the
// session logger (a nil `*logger` for the nop logger), so it is always
// true; the nop logger is `None` here and its calls do nothing.



// Go: project/session.go:1937 shouldPublishProgramDiagnostics


#[cfg(test)]
mod tests {
    use super::*;

    const NOP: Option<Rc<dyn logging::Logger>> = None;

    /// A preempt with a live warm, and the warm's context.
    fn preempt_with_warm() -> (WarmAutoImportPreempt, Context) {
        let preempt = WarmAutoImportPreempt::default();
        let (ctx, cancel) = gostd::context::with_cancel(&gostd::context::background());
        preempt.set(ctx.clone(), cancel, "/a.ts".to_string());
        (preempt, ctx)
    }

    /// Runs `on_message` on another thread, and returns once the message is
    /// queued. The thread gives its wait, from the queue call, and whether it
    /// queued the message. The wait does not count the thread's start, so a
    /// slow start under load does not shorten it.
    fn message(
        preempt: &WarmAutoImportPreempt,
        file_event: bool,
    ) -> std::thread::JoinHandle<(Duration, bool)> {
        let preempt = preempt.clone();
        let (queued_tx, queued_rx) = std::sync::mpsc::channel();
        let reader = std::thread::spawn(move || {
            let mut start = None;
            let queued = preempt.on_message(file_event, &NOP, || {
                start = Some(Instant::now());
                queued_tx.send(()).expect("test thread");
                true
            });
            (start.expect("message queued").elapsed(), queued)
        });
        queued_rx.recv().expect("reader thread");
        reader
    }

    // A message that comes during an eager attempt waits for the clone, at
    // most for the hold. A clone that ends in time is neither cancelled nor
    // yielded. The clone ends 20 ms after the message is queued, so the
    // message waits at least that long, whatever the load.
    #[test]
    fn a_message_waits_for_an_eager_attempt_until_the_clone_ends() {
        let (preempt, warm_ctx) = preempt_with_warm();
        for file_event in [false, true] {
            let (attempt_ctx, attempt_cancel) = gostd::context::with_cancel(&warm_ctx);
            assert!(preempt.start_attempt(attempt_cancel, Some(Duration::from_secs(60))));
            let reader = message(&preempt, file_event);
            std::thread::sleep(Duration::from_millis(20));
            assert_eq!(
                preempt.end_attempt(&warm_ctx, &attempt_ctx),
                WarmAttemptEnd::Done
            );
            let (waited, queued) = reader.join().expect("reader thread");
            assert!(queued);
            assert!(waited >= Duration::from_millis(20), "{waited:?}");
            assert!(waited < Duration::from_secs(60), "{waited:?}");
            assert!(attempt_ctx.err().is_none() && warm_ctx.err().is_none());
        }
    }

    // After the hold, a message other than a file event yields the eager
    // attempt, and a file event cancels the warm.
    #[test]
    fn after_the_hold_a_message_yields_the_attempt_and_a_file_event_cancels_the_warm() {
        let hold = Duration::from_millis(20);
        let (preempt, warm_ctx) = preempt_with_warm();
        let (attempt_ctx, attempt_cancel) = gostd::context::with_cancel(&warm_ctx);
        assert!(preempt.start_attempt(attempt_cancel, Some(hold)));
        let (waited, queued) = message(&preempt, false).join().expect("reader thread");
        assert!(queued && waited >= hold, "{waited:?}");
        assert!(attempt_ctx.err().is_some() && warm_ctx.err().is_none());
        assert_eq!(
            preempt.end_attempt(&warm_ctx, &attempt_ctx),
            WarmAttemptEnd::Yielded
        );

        let (attempt_ctx, attempt_cancel) = gostd::context::with_cancel(&warm_ctx);
        assert!(preempt.start_attempt(attempt_cancel, Some(hold)));
        let (waited, queued) = message(&preempt, true).join().expect("reader thread");
        assert!(queued && waited >= hold, "{waited:?}");
        assert!(warm_ctx.err().is_some());
        assert_eq!(
            preempt.end_attempt(&warm_ctx, &attempt_ctx),
            WarmAttemptEnd::Cancelled
        );
    }

    // During an attempt that is not eager, a message does not wait: a file
    // event cancels the warm at once, and other messages leave it running.
    #[test]
    fn a_message_does_not_wait_for_an_attempt_that_is_not_eager() {
        let (preempt, warm_ctx) = preempt_with_warm();
        let (attempt_ctx, attempt_cancel) = gostd::context::with_cancel(&warm_ctx);
        assert!(preempt.start_attempt(attempt_cancel, None));
        let (_, queued) = message(&preempt, false).join().expect("reader thread");
        assert!(queued && attempt_ctx.err().is_none() && warm_ctx.err().is_none());
        let (_, queued) = message(&preempt, true).join().expect("reader thread");
        assert!(queued && warm_ctx.err().is_some());
        assert_eq!(
            preempt.end_attempt(&warm_ctx, &attempt_ctx),
            WarmAttemptEnd::Cancelled
        );
    }

    // An attempt does not start while a message waits for the dispatch
    // thread.
    #[test]
    fn an_attempt_does_not_start_while_a_message_waits() {
        let (preempt, warm_ctx) = preempt_with_warm();
        let busy = Arc::new(std::sync::atomic::AtomicBool::new(true));
        {
            let busy = busy.clone();
            preempt.set_busy(Box::new(move || {
                busy.load(std::sync::atomic::Ordering::SeqCst)
            }));
        }
        let (_, attempt_cancel) = gostd::context::with_cancel(&warm_ctx);
        assert!(!preempt.start_attempt(attempt_cancel.clone(), Some(Duration::ZERO)));
        busy.store(false, std::sync::atomic::Ordering::SeqCst);
        assert!(preempt.start_attempt(attempt_cancel, Some(Duration::ZERO)));
    }
}
