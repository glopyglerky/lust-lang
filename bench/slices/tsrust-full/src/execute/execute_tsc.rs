//! Go: execute/tsc.go, the `tsc` command line: `CommandLine`,
//! `tscBuildCompilation`, `tscCompilation`, `findConfigFile`,
//! `getTraceFromSys`, `performIncrementalCompilation`, `performCompilation`
//! and `showConfig`. `startTracingIfNeeded` and `stopTracing` write the
//! warnings of the process session in `crate::tracing`.
//!
//! Every bin runs this code. `tsgo` runs it as Go does (`GoTsc`).
//! `goport` and `goport_emit` pass their own `TscCompilationHooks`.
//! `goport_build` runs `tsc_build_compilation`.
//!
//! PORT: Go `ctx` reaches watch mode (execute/watcher.rs) and the build
//! orchestrator. Go `testing` is `TscCompilationHooks::testing` (`None`
//! outside tests, see compile.rs).
//!
//! PORT: a plain `tsc` run has one program for the process, so Go
//! `compiler.NewProgram` is `crate::program::install_new_program` (see
//! PORTING.md "Program"). Call `tsc_compilation` once per process, on the
//! thread that should own the program and the checker pool. A `-b` build
//! makes one program version per project instead (build/build_task.rs). Go
//! catches no panic here; the bins guard unported code.

use crate::frontend::prelude::*;

use std::sync::Arc;
use std::time::SystemTime;

use crate::contentmapper::{
    
    
};
use crate::emitter::program_emit::{};
use crate::execute::build::orchestrator::{};
use crate::execute::incremental::incremental::{};
use crate::execute::incremental::program::{
    NestedEmitNow, 
    
};
use crate::execute::tsc::{
    CommandLineResult, CompilerProgram, 
    ExitStatus, 
    
    
    
};
// PORT: testing


/// Go `tsc`: no bin step, the Go program, and emit writes through the OS
/// file system. `tsgo` runs this.
pub struct GoTsc;



/// `tsc.CommandLineResult{Status: status}`.
fn result(status: ExitStatus) -> CommandLineResult {
    CommandLineResult {
        status,
        watcher: None,
    }
}

/// Ends a run that asks for `option`, which the wasm build cannot run:
/// watch mode needs threads and file events, and `--pprofDir` profiles the
/// process. crates/ts_wasm refuses both options before tsc runs, with the
/// same text and status; a response file (`@file`) can still set them.
/// The gates that call this let the link leave the watch and profile code
/// out of the wasm module.
// PORT: not in Go.
#[cfg(target_family = "wasm")]
fn wasm_unsupported(option: &str) -> CommandLineResult {
    eprintln!("error: {option} is not supported by the wasm build");
    result(ExitStatus::DiagnosticsPresentOutputsSkipped)
}

// Go: execute/tsc.go:28 startTracingIfNeeded, the warning part. The session
// is the process session in `crate::tracing`.
// PORT: testing. `testing` is Go `testing != nil`.

// Go: execute/tsc.go:44 stopTracing

// Go: execute/tsc.go:53 CommandLine
// PORT: Go parses the build command line here and passes it on; the port
// passes the arguments, because `goport_build` calls
// `tsc_build_compilation` with them too. `hooks.build_mode` and
// `hooks.command_line_parsed` are the bin's steps (not in Go).

// Go: execute/tsc.go:66 fmtMain
// PORT: not ported. Its only call (the `-f` case in `CommandLine`) is
// commented out in Go, so it is dead code, and the formatter is not ported.

// Go: execute/tsc.go:91 tscBuildCompilation
// PORT: Go `CommandLine` parses the build command line and passes it in;
// here it is the first step, which runs in the same order.
// `command_line_args` is the full command line (Go `commandLineArgs`).
// PORT: the orchestrator compiles every project in this process, on this
// thread (build/build_task.rs).

// Go: execute/tsc.go:122 tscCompilation
// PORT: `hooks.prepare_compilation` is the bin's step (not in Go).

// Go: execute/tsc.go:269 findConfigFile

// Go: execute/tsc.go:283 getTraceFromSys




// Go: execute/tsc.go:287 performIncrementalCompilation
// PORT: the incremental program reads back the installed program. A bin
// that replaces the program (`TscCompilationHooks::program_like`) skips
// `incremental.ReadBuildInfoProgram` and `incremental.NewProgram`: `goport`
// and `goport_emit` run on read-only Query inputs, which are composite and
// incremental, and the Go incremental program would write build info next
// to them. Their steps are still timed (empty), so the statistics table
// has the same rows as Go. Without the incremental program there is no
// `testing.OnProgram` (only a bin replaces it, and bins pass no testing).
// PORT: perf. The incremental program sends its check, and when it can,
// the emit, before
// `EmitAndReportStatistics` (`Program::start_check_and_emit`). Each checker
// then emits when its own check ends. `EmitAndReportStatistics` makes the
// same calls as in Go and waits for that work. Its check time is the time
// that `start_check` spent on the affected files plus the wait for the
// check (`Program::take_started_check_time`), and its emit time the wait
// for the rest of the emit.
// PORT: `sys_rc` is the `Rc` so that the incremental program can keep
// `sys.Now` (Go passes the method value) and the content mapper host can
// keep `sys` as its spawner. The body uses `sys: &dyn System`.

// Go: execute/tsc.go:345 performCompilation
// PORT: `sys_rc` is the `Rc` so that the content mapper host can keep
// `sys` as its spawner. The body uses `sys: &dyn System`.




// Go: execute/tsc.go:394 getContentMapperProject (tsgo#4712)

// Go: execute/tsc.go:405 showConfig

// PORT: added for `GoTsc`. Go passes no `WriteFile`, so `emitHost.WriteFile`
// (compiler/emitHost.go:120) writes through `program.Host().FS()`
// (cachedvfs over bundled over osvfs). The port's emit host refuses a
// write without a callback (program.rs emitHost `write_file`), and emit
// runs on the checker threads, so the callback must be `Send` and cannot
// hold the `Rc` file system. Both wrappers pass a write of a real path to
// osvfs (cachedvfs.go:144 WriteFile), so this writes with the osvfs of the
// calling thread, like `new_task_write_file` in build/build_task.rs without
// the build info tracking. There is no outDir or input guard: tsgo writes
// next to the sources or into outDir, as Go does.
