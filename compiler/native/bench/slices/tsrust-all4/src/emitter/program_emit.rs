//! Port of the emit parts of Go `compiler/program.go` (`Emit`,
//! `CombineEmitResults`, `HandleNoEmitOptions`, the emit option and result
//! types). The emit host (`program::EmitHost`) and the output paths
//! (`program::get_output_paths_for_source_file`) are in `program.rs`, where
//! the declaration diagnostics use them too.

use crate::prelude::*;

use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

use super::emitter::{EmitOnly, };
use crate::frontend::outputpaths::{ForceEmitPaths, OutputPaths};
use crate::frontend::tspath::{Path, path_is_relative, };
use crate::sourcemap::generator::RawSourceMap;

// Go: compiler/program.go:1837 WriteFileData

// Go: compiler/program.go:1845 WriteFile
// PORT: Go `error` is `Result<(), String>`. Emit runs on the checker
// threads and the emit pool, so the callback is shared and thread safe. The
// callback may set `skipped_dts_write`.

// Go: compiler/program.go:1847 EmitOptions
// PORT: Go `TargetSourceFiles` nil is `None`. A Go non-nil empty slice is
// `Some` of an empty list, which emits no file (#4699).



// Go: compiler/program.go:1854 EmitResult

// Go: compiler/program.go:1861 SourceMapEmitResult
#[derive(Clone, Debug, Default)]
pub struct SourceMapEmitResult {
    /// Input source file (which one can use on program to get the file), 1:1 mapping with the sourceMap.sources list
    pub input_source_file_names: Vec<String>,
    pub source_map: RawSourceMap,
    pub generated_file: String,
}

// Go: compiler/program.go:1867 Program.Emit
// PORT: Go queues one emit per file on a work group and takes a writer from
// a pool. Each file's emit runs on the thread of its checker (the emit
// resolver reaches that checker there), and the results combine in file
// order. Each emit makes its own text writer. When the emit pool is on
// (`program::emit_pool_enabled`), the JS part of a file whose transforms
// make no checker call runs on the pool instead, and the prints of the
// parts that run on a checker go to its twin (`start_emit_files_with_pool`).
// Go `ctx.Err()` checks are not ported: the port has no context here.
















/// False when `GOPORT_EARLY_EMIT` is `0`: the emit then waits for the whole
/// check, as in Go (`emit_can_start_with_check`). For tests and timing on
/// one binary. Read once.
fn early_emit_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("GOPORT_EARLY_EMIT").as_deref() != Ok("0"))
}




/// Where one file's emit runs when the emit pool is on (`file_emit`).
#[derive(Clone, Copy, PartialEq, Eq)]
enum FileEmit {
    /// All of it on the file's checker thread, as with the pool off.
    OnChecker,
    /// The JS part on the emit pool, the d.ts part on the checker thread.
    Split,
    /// All of it on the emit pool: it has a JS part and no d.ts part.
    OnPool,
}

















/// Where the parts whose transforms run on a checker thread print: the d.ts
/// part of a split file, and the JS and d.ts parts of a file whose JS part
/// needs the checker (`GOPORT_DTS_TWIN`, `GOPORT_DTS_TWIN_CHECK`,
/// `set_dts_twin_mode`). The twins are on only with the emit pool
/// (`program::emit_pool_enabled`: not with `--singleThreaded` or a trace).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DtsTwinMode {
    /// On the checker thread, after its transforms.
    Off,
    /// On the twin of the checker (`program::send_dts_twin_job`).
    On,
    /// On the twin, and on the checker too: the twin panics when its text
    /// differs from the checker's (tests and checks).
    Check,
}

/// The mode that `set_dts_twin_mode` set: 0 for none, else the mode + 1.
static DTS_TWIN_MODE_SET: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

/// Sets the d.ts twin mode of this process, or with `None` goes back to the
/// environment (`dts_twin_mode`). Tests use it.
pub fn set_dts_twin_mode(mode: Option<DtsTwinMode>) {
    let value = match mode {
        None => 0,
        Some(DtsTwinMode::Off) => 1,
        Some(DtsTwinMode::On) => 2,
        Some(DtsTwinMode::Check) => 3,
    };
    DTS_TWIN_MODE_SET.store(value, std::sync::atomic::Ordering::Relaxed);
}

/// PORT: not in Go (perf). The d.ts twin mode: `set_dts_twin_mode`, else
/// `Off` when `GOPORT_DTS_TWIN=0`, `Check` when `GOPORT_DTS_TWIN_CHECK=1`,
/// else `On`. The environment is read once.
pub fn dts_twin_mode() -> DtsTwinMode {
    static ENV: OnceLock<DtsTwinMode> = OnceLock::new();
    match DTS_TWIN_MODE_SET.load(std::sync::atomic::Ordering::Relaxed) {
        1 => DtsTwinMode::Off,
        2 => DtsTwinMode::On,
        3 => DtsTwinMode::Check,
        _ => *ENV.get_or_init(|| {
            if std::env::var("GOPORT_DTS_TWIN").as_deref() == Ok("0") {
                DtsTwinMode::Off
            } else if std::env::var("GOPORT_DTS_TWIN_CHECK").as_deref() == Ok("1") {
                DtsTwinMode::Check
            } else {
                DtsTwinMode::On
            }
        }),
    }
}


/// The kind of tree that a `TwinPrint` prints.
enum TwinTree {
    /// A JS file (`JsPrint`).
    Js,
    /// A d.ts file (`DeclarationPrint`).
    Declaration {
        /// `DeclarationPrint::content_mapped_source`: a parsed SourceFile,
        /// which every thread reads.
        content_mapped_source: Node,
        emit_declaration_map: bool,
    },
}

/// The JS prints sent to twins in this process (`js_twin_print_count`).
static JS_TWIN_PRINTS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// The number of JS prints sent to twins in this process so far. Tests use
/// it to see that the twins printed JS files.
pub fn js_twin_print_count() -> usize {
    JS_TWIN_PRINTS.load(std::sync::atomic::Ordering::Relaxed)
}










// Go: compiler/program.go:1939 CombineEmitResults

// Go: compiler/program.go:1976 HandleNoEmitOptions
// HandleNoEmitOptions mirrors tsc's handleNoEmitOptions.
// PORT: #4407 replaced Go `HandleNoEmitOnError`. This is the plain program
// form, for `Program.Emit`, which passes a nil `emitBuildInfo`, so it has no
// such parameter. The incremental program has its own form
// (`execute::incremental::program`).
