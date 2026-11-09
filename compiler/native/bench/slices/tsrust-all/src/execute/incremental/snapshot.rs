//! Port of execute/incremental/snapshot.go from line 133 (after
//! `emitSignature`, which is in `hash.rs`): the build info diagnostics with
//! file names, the diagnostics repopulation, and the `snapshot` state.
//!
//! PORT: Go `*compiler.Program` parameters are dropped and the current
//! program (`prog()`) is read through the `program.rs` free functions. Go
//! `SyncMap` and `SyncSet` fields are `FxIndexMap` and `FxIndexSet` (see
//! `reference_map.rs` for the order note); Go `atomic.Bool` and `sync.Once`
//! become plain fields, because the snapshot is only used on the loading
//! thread.
//!
//! PERF: the map keys are long paths, so the maps use the Fx hasher instead
//! of SipHash. The hasher does not change the order: it is insertion order.

use super::hash::FileInfo;
use super::hash::*;
use crate::frontend::prelude::*;
use std::cell::OnceCell;

/// Go `*ast.RepopulateDiagnosticInfo`, as `Diagnostic::repopulate_info`
/// returns it (`Arc`, because a `Diagnostic` crosses the checker threads).
pub type RepopulateInfoRef = std::sync::Arc<RepopulateDiagnosticInfo>;

// Go: incremental/snapshot.go:133 buildInfoDiagnosticWithFileName
// PORT: Go `diagnostics.Category` is kept as the raw Go value, like
// `BuildInfoDiagnostic.category` (`diagnostics::Category(raw)` gives the Go
// value). The Go slices are `Option`s, because the build info leaves out a
// nil slice but writes an empty one (`omitzero`, buildInfo.go:209 to :211).
// A read empty list stays empty (`core.Map`, buildinfotosnapshot.go:85 to
// :87), so it is written again as `[]` (snapshottobuildinfo.go:140 to
// :142).
#[derive(Clone, Debug, Default)]
pub struct BuildInfoDiagnosticWithFileName {
    // filename if it is for a File thats other than its stored for
    pub file: Path,
    pub no_file: bool,
    pub pos: i32,
    pub end: i32,
    pub code: i32,
    pub category: i32,
    pub source: String,
    pub message_text: String,
    pub message_key: String,
    pub message_args: Option<Vec<String>>,
    pub message_chain: Option<Vec<BuildInfoDiagnosticWithFileName>>,
    pub related_information: Option<Vec<BuildInfoDiagnosticWithFileName>>,
    pub reports_unnecessary: bool,
    pub reports_deprecated: bool,
    pub skipped_on_no_emit: bool,
    pub repopulate_info: Option<RepopulateInfoRef>,
}



// Go: incremental/snapshot.go:153 DiagnosticsOrBuildInfoDiagnosticsWithFileName
// PORT: Go nil `diagnostics` is `None`; it marks "not converted yet".
// PORT: testing. Go maps hold pointers to these, and the Go test harness
// compares the old and new program's pointers (tsctests/sys.go OnProgram).
// `id` is that identity: each new entry (Go
// `&DiagnosticsOrBuildInfoDiagnosticsWithFileName{...}`, here `default()`
// or `..Default::default()`) gets a new id, and a clone (Go keeps the
// pointer) keeps it. No output reads it.

// PORT: testing (see `DiagnosticsOrBuildInfoDiagnosticsWithFileName`)

/// PORT: testing. A new `DiagnosticsOrBuildInfoDiagnosticsWithFileName` id.
pub fn new_diagnostics_id() -> u64 {
    static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

impl BuildInfoDiagnosticWithFileName {
    // Go: incremental/snapshot.go:158 toDiagnostic

    // Go: incremental/snapshot.go:212 toDiagnosticWithoutRepopulate
}

// Go: incremental/snapshot.go:199 repopulateDiagnosticChain
// repopulateDiagnosticChain recomputes a diagnostic chain entry that depends on
// program state which may have changed between incremental builds.

// Go: incremental/snapshot.go:236 repopulateModeMismatchChain

// Go: incremental/snapshot.go:263 repopulateModuleNotFoundChain


// Go: incremental/snapshot.go:306 snapshot
// PORT: Go `options *core.CompilerOptions` is `&'static`: the program
// options are static, and `buildInfoToSnapshot` leaks the options it reads
// (one per process). `compiler.ProgramLike.Options` needs `&'static`.


// Go: incremental/snapshot.go:389 getTextHandlingSourceMapForSignature

// Go: incremental/snapshot.go:396 computeSignatureWithDiagnostics
// PORT: the body of the Go method, as a free function over `hashWithText`,
// so emit `WriteFile` callbacks on the checker threads can call it without
// the snapshot.

// Go: incremental/snapshot.go:405 diagnosticToStringBuilder
