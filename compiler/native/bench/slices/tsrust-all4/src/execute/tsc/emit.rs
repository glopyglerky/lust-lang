//! Go: execute/tsc/emit.go (emit a program and report its diagnostics).
//!
//! PORT: Go `compiler.ProgramLike` (compiler/program.go:1710) is the
//! `ProgramLike` trait below, because this is its first user in the port.
//! `CompilerProgram` is the plain `*compiler.Program` over the current
//! program (`prog()`); the incremental program implements the same trait.
//! Go `EmitInput.Program` (the underlying `*compiler.Program`) is the
//! current program and is not a field.

use crate::prelude::*;

use crate::emitter::program_emit::{};
use crate::locale::Locale;

use super::compile::{ExitStatus, System, Writer, write_str};
use super::diagnostics::{};
use super::statistics::{};

// PORT: testing (`CommandLineTesting`)
use crate::frontend::tspath::Path;
use std::sync::Mutex;
use std::time::SystemTime;

// Go: compiler/program.go:1957 ProgramLike
// PORT: only the methods that `EmitFilesAndReportErrors` and
// `GetDiagnosticsOfAnyProgram` call. Config, syntactic and program
// diagnostics are read from the current program by
// `get_diagnostics_of_any_program`; Go `incremental.Program` forwards them
// to its program too. Go passes a context; the port has none. The methods
// take `&self` like Go interface methods; an implementation with state
// (the incremental snapshot) uses interior mutability.

/// Go `*compiler.Program` as a `ProgramLike`: the current program.
#[derive(Clone, Copy, Debug, Default)]
pub struct CompilerProgram;


// Go: execute/tsc/emit.go:21 GetTraceWithWriterFromSys

// Go: execute/tsc/emit.go:31 EmitInput
// PORT: `Program` is the current program (see the module comment).
// `Testing` and `TestingMTimesCache` are `None` outside Go tests; the cache
// is the build host `m_times` (a `Mutex`, Go `SyncMap`). `Tracing` is the
// process session (`crate::tracing::get`). Go `Config` is optional here:
// without it, the program options are used, which are the config options
// with the command line applied.


// Go: execute/tsc/emit.go:46 EmitAndReportStatistics

/// Go `input.Sys.Now().Sub(start)`. A clock that goes back gives zero.
fn since(sys: &dyn System, start: SystemTime) -> std::time::Duration {
    sys.now().duration_since(start).unwrap_or_default()
}

// Go: execute/tsc/emit.go:74 EmitFilesAndReportErrors
// PORT: Go times each bind, check and emit call with `sys.Now()`, and so
// does the port. When the incremental program started the check before
// (`Program::start_check`, `tsc -p` and `tsc -b`), the check time adds the
// time of that start (`Program::take_started_check_time`), and the check
// call times the wait for the rest.

// Go: execute/tsc/emit.go:144 listFiles
// PORT: Go `fmt.Fprintln(w, "TSFILE:", x)` puts a space between operands.
