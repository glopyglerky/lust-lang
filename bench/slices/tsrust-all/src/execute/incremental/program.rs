//! Port of execute/incremental/program.go, plus `ReadBuildInfoProgram`
//! (incremental.go:43), which builds a `Program` from a snapshot.
//!
//! PORT: Go `Program.program` is the current `&'static GoProgram`
//! (`prog()`, nil for a program read from build info), and its methods are
//! the `program.rs` free functions, which read `prog()`. A caller with
//! several programs (the build task) makes the program current while it
//! uses this one. Go passes a context; the port has none. The
//! `ProgramLike` methods take `&self`, so the snapshot is behind a
//! `RefCell`; it is an `Rc` because Go `programToSnapshot` can reuse the
//! old program's snapshot.

use super::build_info::*;
use super::checker_access::*;
use super::emit_files::{
    
};
use super::hash::FileInfo;
use super::incremental::{};
use super::snapshot::*;
use crate::emitter::emitter::EmitOnly;
use crate::emitter::program_emit::{
    
};
use crate::frontend::prelude::*;
use crate::gostd::slices::stable_sort_by;
use std::cell::Cell;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// Go `func() time.Time`, the `nestedEmitNow` of `NewProgram` (`sys.Now`).
pub type NestedEmitNow = Rc<dyn Fn() -> SystemTime>;

// Go: incremental/program.go:23 SignatureUpdateKind
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum SignatureUpdateKind {
    #[default]
    ComputedDts = 0,
    StoredAtEmit = 1,
    UsedVersion = 2,
}

// Go: incremental/program.go:31 Program
// PORT: Go `host` is nil for a program read from build info. Go
// `nestedEmitMu` guards the nested emit fields; the program is used on one
// thread, so they are `Cell`s. Go `time.Time` is `Option<SystemTime>`
// (`None` = zero).


// Go: incremental/program.go:48 NewProgram
// PORT: Go `program` is the current program (`prog()`), so it is not a
// parameter.

// Go: incremental/incremental.go:44 ReadBuildInfoProgram


// Go: incremental/program.go:69 TestingData
// PORT: testing. Go keeps pointers to the new snapshot's and the old
// program's `semanticDiagnosticsPerFile`, and the Go test harness compares
// the entry pointers. The entry identity is its `id` (see
// `DiagnosticsOrBuildInfoDiagnosticsWithFileName`). Go
// `SemanticDiagnosticsPerFile` is `Program::semantic_diagnostics_id`,
// which reads the current snapshot as the Go pointer does. Go
// `OldProgramSemanticDiagnosticsPerFile` is `old_semantic_diagnostics_ids`,
// the old map's ids when `new_program` ran (empty without an old program).
// Nothing changes the old map after that, except when Go
// `programToSnapshot` reuses the old snapshot (same program). Then Go
// compares one map with itself; only watch mode (not on this branch) makes
// that case, and it must read the ids live.
#[derive(Clone, Debug, Default)]
pub struct TestingData {
    pub updated_signature_kinds: FxHashMap<Path, SignatureUpdateKind>,
    pub old_semantic_diagnostics_ids: FxHashMap<Path, u64>,
}


// Go: incremental/program.go:477 normalizePackageJsons
// PORT: Go returns a new empty slice for nil. The list is sorted, so
// `dedup` gives the same result as Go `core.Deduplicate`.

// Go: compiler/program.go:1976 HandleNoEmitOptions
// HandleNoEmitOptions mirrors tsc's handleNoEmitOptions.
// PORT: #4407 replaced Go `HandleNoEmitOnError`.
// `emitter::program_emit::handle_no_emit_options` is the plain program form
// (nil `emitBuildInfo`). Go passes the `ProgramLike`, whose bind and
// semantic diagnostics are the incremental ones here, so this is the same
// body over `ProgramLike`. Go `files` nil is `None`.

// Go: compiler/program.go:1957 ProgramLike (var _ compiler.ProgramLike = (*Program)(nil))
