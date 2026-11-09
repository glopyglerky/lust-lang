use crate::contentmapper;
use crate::emitter::program_emit::{};
use crate::execute::build::host::{WrittenPaths};
use crate::execute::build::up_to_date_status::*;
use crate::execute::incremental::build_info::{
    
    
};
use crate::execute::incremental::emit_files::{};
use crate::execute::incremental::program::{
    NestedEmitNow, 
    
};
use crate::execute::incremental::{};
use crate::execute::tsc::compile::{};
use crate::execute::tsc::diagnostics::{
    
};
use crate::execute::tsc::emit::{
    
};
use crate::execute::tsc::{ExitStatus, };
use crate::frontend::prelude::*;
use crate::gostd::GoError;
// PORT: testing
use std::sync::{Arc, Mutex, PoisonError};
use std::time::SystemTime;

// This file ports execute/build/buildtask.go.
//
// PORT: concurrency. `ParsedCommandLine` and the frontend program are not
// `Send`, so tasks are `Rc<RefCell<BuildTask>>` and run on one thread, the
// orchestrator thread. It is the loading thread of every program of the
// build. Go `done` and `built` channels and the mutexes are dropped.
// `buildProject` is split where `compileAndEmit` has made the program
// (`build_project_start`) and the rest
// (`build_project_finish`). The orchestrator starts a task after its
// upstream tasks are done, finishes it later, and calls `report` in
// `order` (see orchestrator.rs). The work that Go does on the task
// goroutines still runs at the same time: each program checks on its own
// checker threads from `build_project_start` on (`start_check`), emits on
// them when the check ends (the writes wait for `build_project_finish`),
// and frees them in the background (`release_task_program`).
//
// PORT: the watch-only `updateWatch` and `resetConfig` are in
// orchestrator_watch.rs, with the orchestrator watch code.
//
// PORT: the task keeps the statistics of `tsc.EmitAndReportStatistics` for
// the build aggregate, as Go does. `opts.Testing` is
// `BuildTaskOrchestrator::testing` (`None` outside tests).
//
// PORT: Go `time.Time` is `Option<SystemTime>` (`None` = zero), as in
// up_to_date_status.rs.

// Go: build/buildtask.go:25 buildKind
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BuildKind {
    #[default]
    None,
    Pseudo,
    Program,
}

// Go: build/buildtask.go:33 upstreamTask
// PORT: Go `int` index into `ProjectReferences()` is `usize`.

// Go: build/buildtask.go:37 buildInfoEntry
// PORT: Go `*time.Time` is `Option<Option<SystemTime>>` (nil pointer vs a
// pointer to a possibly zero time). Go `*incremental.BuildInfo` is
// `Option<Rc<BuildInfo>>`.


/// PORT: not in Go (perf). The options of a task that decide whether its
/// up-to-date check returns before it reads the mtimes of its inputs. A
/// prefetch thread reads for the check only what the check reads
/// (`reads_input_times`).
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct StatusCheckOptions {
    is_incremental: bool,
    emit_declarations: bool,
    no_check: bool,
    no_emit: bool,
    /// The Effect rules run (`BuildInfo::is_valid_version`).
    effect: bool,
}

impl StatusCheckOptions {

}



// Go: tsc/diagnostics.go:23 DiagnosticReporter, bound to the task's writer.
// PORT: Go reporters capture `&t.result.builder` as their `io.Writer`. A
// Rust closure cannot hold that borrow while the task also writes to the
// builder, so the reporter takes the writer as its first argument. The
// orchestrator wraps the tsc reporters (`CreateBuilderStatusReporter`,
// `CreateDiagnosticReporter`) into this shape.

// Go: build/buildtask.go:44 taskResult
// PORT: Go `program *incremental.Program` is `program` (`None` = nil). In
// tests the task keeps it until it reports, for `Testing.OnProgram`, and
// the orchestrator releases it there (`release_task_program`), where Go
// drops `t.result`. Outside tests the orchestrator releases it when the
// task is built, where Go sets it to nil (ts#64220).
// `has_changed_dts_file` is its `HasChangedDtsFile()` after the emit,
// which `updateDownstream` reads. Go `*tsc.Statistics` is an `Option`
// (nil = `None`).



/// PORT: not in Go (perf). An optional value that frees on the thread of
/// `drop_in_background` when it drops.
struct DropInBackground<T: Send + 'static>(Option<T>);

impl<T: Send + 'static> std::ops::Deref for DropInBackground<T> {
    type Target = Option<T>;
    fn deref(&self) -> &Option<T> {
        &self.0
    }
}




// The parts of Go `*Orchestrator` (and its `host`) that a build task uses.
// The orchestrator (orchestrator.rs) implements this.

// Go: build/buildtask.go:55 BuildTask
// PORT: Go `*tsoptions.ParsedCommandLine` shared with the host cache is
// `Option<Rc<ParsedCommandLine>>`. Go `*upToDateStatus` is
// `Option<UpToDateStatus>`. `pending` is a plain bool (see top). Go
// `contentMapperProjectOnce` (a `sync.Once`) is a bool; Go nil project and
// error are `None`.

/// PORT: a Go panic in a builder goroutine of Go `rangeTasks`. In Go the
/// report goroutine still writes the reports of the tasks that are built
/// before the panic ends the process. The port reports on the thread that
/// runs the tasks, so `build_project_start` keeps the panic and `report`
/// raises it: the tasks before this one in `order` report first.
enum TaskGoPanic {
    /// The task panicked with this `go_panic` payload.
    Panicked(Box<dyn std::any::Any + Send>),
    /// An upstream task panicked. Go's builder of this task waits on it
    /// (`waitOnUpstream`) until the process ends, so the task does nothing.
    Upstream,
}

// The state of Go `compileAndEmit` from `NewProgram` to
// `EmitAndReportStatistics`: the program is made and not emitted yet.



// Go: build/buildtask.go:621 isContentMapperSupplementalBuildInfoPath (tsgo#4712)
// PORT: Go `strconv.Atoi(index) == nil` is `index.parse::<i64>().is_ok()`:
// both take an optional sign and decimal digits, and fail on an empty
// text and on overflow of a 64-bit int.


// Go: build/buildtask.go:906 (*BuildTask).writeFile
// PORT: emit writes the source outputs on the checker threads, so the
// callback is `Send` and cannot hold the task, the `Rc` system or the `Rc`
// file system. Go writes through `orchestrator.host.FS()` (cachedvfs over
// bundled over osvfs); both wrappers pass a write of a real path to osvfs
// (cachedvfs.go:144 WriteFile), so this writes with the osvfs of the
// calling thread. The build info branch keeps what `onBuildInfoEmit` needs
// in `written`, and `compile_and_emit_finish` calls it after the emit. The
// watch-only `storeMTime` branch stores into the build host `m_times` at
// once (Go `SyncMap`), before the test `OnEmittedFiles` reads it.
// With `deferred` (`System::emit_writes_through_osvfs` is false) it writes
// through the system file system instead (see `DeferredWrites`). Each
// written path goes into `written_paths`, the build host's `written` (not
// in Go).

/// PORT: not in Go. The emit writes of a task whose system file system only
/// the orchestrator thread can reach (`System::emit_writes_through_osvfs`,
/// the API build orchestrator). Go's `writeFile` writes through
/// `orchestrator.host.FS()` from the emit goroutines. Here a write on the
/// orchestrator thread (the build info, and the early emit's writes that
/// `finish_emit_files` flushes there) goes to that file system at once, after
/// the waiting ones. A write on another thread (a checker thread) waits in
/// `writes`, and the next write on the orchestrator thread, or the end of the
/// task's emit, makes it. A waiting write that fails is kept in `failed` for
/// Go's TS5033 after the emit (`compile_and_emit_finish`).
#[derive(Default)]
struct PendingWrites {
    writes: Vec<(String, String)>,
    failed: Vec<(String, String)>,
}

type DeferredWrites = Arc<Mutex<PendingWrites>>;




// Go `orchestrator.opts.Sys.Now()` in the task `writeFile`.
// PORT: emit writes the source files' outputs on the checker threads,
// which cannot hold the `Rc` system; there it is the OS system's `Now`
// (`SystemTime::now`). The build info write runs on the orchestrator thread
// (incremental `emitBuildInfo`), so its time comes from the orchestrator
// system, in the same order as Go with the test `OnEmittedFiles` times.


#[cfg(test)]
mod tests {
    use super::*;

    // Go: build/buildtask_contentmapper_test.go:11 TestIsContentMapperSupplementalBuildInfoPath (tsgo#4712)
    #[test]
    fn test_is_content_mapper_supplemental_build_info_path() {
        let roots = [
            Path("/src/app.vue".to_string()),
            Path("/src/index.ts".to_string()),
        ];
        let path = |text: &str| Path(text.to_string());

        assert!(is_content_mapper_supplemental_build_info_path(
            &path("/src/app.vue.0.ts"),
            roots.iter()
        ));
        assert!(is_content_mapper_supplemental_build_info_path(
            &path("/src/app.vue.12.mts"),
            roots.iter()
        ));
        assert!(!is_content_mapper_supplemental_build_info_path(
            &path("/src/app.vue.ts"),
            roots.iter()
        ));
        assert!(!is_content_mapper_supplemental_build_info_path(
            &path("/src/app.vue.0.txt"),
            roots.iter()
        ));
        assert!(!is_content_mapper_supplemental_build_info_path(
            &path("/src/other.vue.0.ts"),
            roots.iter()
        ));
    }
}
