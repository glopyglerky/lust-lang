//! Go: execute/build/orchestrator.go, and `tscBuildCompilation` of
//! execute/tsc.go:90 (the `tsc -b` entry).
//!
//! The watch part (`Watch`, `updateWatch`, `resetCaches`,
//! `checkTasksForEventChanges`, `computeDesiredWatches`, `DoCycle`) is in
//! orchestrator_watch.rs.
//!
//! PORT: concurrency. Go runs `buildOrCleanProject` for the tasks of the
//! build order (`order`, or the part of it that an API build asks for,
//! ts#64158) on up to `numRoutines` goroutines (`rangeTasks`); each
//! goroutine takes the next task, waits for its upstream tasks, builds, and
//! closes the task's `built` channel. One more goroutine reports the tasks
//! in the same order, each when it is built (ts#64220). Go computes
//! `scheduleOrder` with the graph, but since ts#64158 the builders take the
//! tasks in build order. Tasks here are
//! `Rc<RefCell<BuildTask>>` on this thread, which makes, emits and releases
//! the program of every task (see build_task.rs). `build_all_tasks` keeps
//! the Go schedule: at most `numRoutines` tasks are taken and not yet
//! built, and tasks are taken and report in build order.
//! A taken task starts when its upstream tasks are done, and a task that
//! compiles makes its program at once (`build_project_start`) and starts
//! its check on the program's checker threads, so the checkers of the
//! started tasks work at the same time, as the Go goroutines do. Each
//! checker emits when its check ends (a task that checks nothing, as with
//! cached semantic diagnostics, `noCheck` or a syntax error, emits at
//! once), and the emit keeps its writes in
//! memory. The started tasks write their outputs one at a time
//! (`build_project_finish`), in the order their check and emit end, as each
//! Go builder writes when its own task ends. PORT (determinism): when tasks
//! can see each other's writes (shared_outputs.rs), all tasks finish in
//! build order instead. Tasks start before a started task writes, so a task
//! that runs beside others in Go reads the file system before they write
//! their outputs. Every task uses `o.host` and its caches (parsed `.d.ts` and
//! `.json` files, configs, the cached file system, the mtimes), as in Go.
//! Outside tests each program is released when its task is built, as Go
//! drops it there; in tests when its task reports. Its checker threads
//! free it in the background. Where Go does task work on its goroutines
//! that needs no task state, threads do it ahead of this thread: the file
//! name match of each config, and the build info read, its check parts
//! and the source mtimes of each task (config_prefetch.rs,
//! `BuildInfoPrefetch`).
//!
//! PORT: the task keeps its project statistics (see build_task.rs), and
//! `report_task` adds them to the aggregate `--diagnostics` and
//! `--extendedDiagnostics` statistics.
//!
//! PORT: testing. `opts.testing` is `None` outside tests. A test compiles
//! each started task to the end at once, in build order, so the one test
//! file system and clock see one ordered sequence (see `build_all_tasks`).

use crate::contentmapper;
use crate::execute::build::build_task::*;
use crate::execute::build::config_prefetch::{
    BuildInfoRead, 
};
use crate::execute::build::shared_outputs::{};
use crate::execute::incremental::build_info::{};
use crate::execute::tsc::compile::{
    CommandLineResult, ExitStatus, Watcher, 
};
use crate::execute::tsc::diagnostics::{
    
    
};
use crate::execute::watchmanager::{};
use crate::frontend::prelude::*;
// PORT: testing
use std::collections::VecDeque;
use std::sync::{Arc, PoisonError};
use std::time::SystemTime;

// Go: build/orchestrator.go:27 Options

// Go: build/orchestrator.go:32 OrchestratorResult (ts#64158)
// PORT: Go `FilesToDelete` is nil until a task adds a file, so an empty
// `Vec` is the Go nil.


// Go: build/orchestrator.go:66 Orchestrator
// PORT: Go `*SyncMap` of tasks is a plain map; tasks are
// `Rc<RefCell<BuildTask>>`. Go `wm *watchmanager.WatchManager` is
// `Rc<RefCell<WatchManager>>`, so the watch loop can run while `DoCycle`
// borrows the orchestrator (see orchestrator_watch.rs).


// PORT: Go `getWriter(task)` gives the reporter `&task.result.builder`. A
// `TaskDiagnosticReporter` gets the builder on each call instead (see
// build_task.rs), so the tsc reporter writes into its own buffer, and the
// buffer moves into the builder after each call.



/// The most released programs that `Orchestrator::released` keeps.
const MAX_KEPT_RELEASED: usize = 4;

/// PORT: not in Go (perf). Sends the build order index of its task when it
/// drops (see `build_all_tasks`).
struct ReadySignal {
    index: usize,
    ready: std::sync::mpsc::Sender<usize>,
}

impl Drop for ReadySignal {
    fn drop(&mut self) {
        let _ = self.ready.send(self.index);
    }
}


// Go: build/orchestrator.go:991 NewOrchestrator
