//! Go `*compiler.Program` checker and diagnostics methods for the language
//! service, plus the Go compiler checker pool (`compiler/checkerpool.go`).
//!
//! Everything here runs on the LSP dispatch thread. Go `*compiler.Program`
//! is `Rc<NewProgram>` where it is stored and `&NewProgram` where code only
//! reads it. A holder of the `Rc` keeps the program alive, as a Go pointer
//! does. A Go program method `p.X(..)` that the language service needs is
//! `ls_program::x(p, ..)`.
//!
//! Programs: each `NewProgram` made by `new_program` or `update_program` is
//! also a program version of the process (`program::new_program_version`),
//! as Go makes a new `Program` for each snapshot change. Versions share the
//! file versions they have in common. The checkers of every version are
//! made here, on the dispatch thread.
//!
//! Current program: the checker and the `program.rs` functions that it
//! calls read `prog()`. `enter` makes a program current while a
//! `ProgramGuard` lives. A language service holds a guard for its program,
//! a checker from `get_type_checker*` holds one until its release, and the
//! functions here enter the program they are given. Guards can drop in any
//! order: the current program is the one of the last guard that is still
//! alive.
//!
//! Release: Go frees a program when no snapshot and no request uses it.
//! `release_program` is the snapshot part (Go `programCounter.Deref`); the
//! release waits until no guard and no hold (`hold_program`) of the program
//! is alive. It frees the version's program tables, and the registry drops
//! its `Rc`, so the `NewProgram` is freed with its last holder. On the
//! dispatch thread of the LSP server both frees wait until the answer is
//! sent (`gostd::local::drop_later`). The `GoProgram` shell and the
//! file versions stay leaked (see `program::release_program`). A compiler
//! host whose last live program is released drops its data
//! (`CompilerHost::release`), even when a stale holder keeps the program.
//!
//! PORT: Go keeps the checker pools in `Program` fields. `NewProgram` does
//! not have them, so they live in a thread-local registry keyed by the
//! program address (`ProgramCheckers`).
//!
//! The compile path (`program::with_type_checker_for_file`, the worker
//! pool) is separate and does not change.

use super::*;
use crate::emitter::emitter::{EmitOnly, };
use crate::emitter::program_emit::{};
use crate::frontend::compiler::{};
use crate::frontend::module;
use crate::frontend::outputpaths::ForceEmitPaths;
use crate::frontend::tspath;
use crate::gostd::Context;
use std::cell::{Cell, OnceCell};

// ---------------------------------------------------------------------------
// Release, CheckerPool
// ---------------------------------------------------------------------------

/// Go `func()` that releases a checker (the `done` of
/// `GetTypeCheckerForFile`). It runs once: on `call` or on drop, whichever
/// comes first.
// PORT: Go wraps release functions in `sync.OnceFunc`.
pub struct Release(Option<Box<dyn FnOnce()>>);

impl Release {
    pub fn new(f: impl FnOnce() + 'static) -> Release {
        Release(Some(Box::new(f)))
    }

    // Go: compiler/checkerpool.go:493 noop
    pub fn noop() -> Release {
        Release(None)
    }

    pub fn call(mut self) {
        if let Some(f) = self.0.take() {
            f();
        }
    }
}

impl Drop for Release {
    fn drop(&mut self) {
        if let Some(f) = self.0.take() {
            f();
        }
    }
}

// Go: compiler/checkerpool.go:22 CheckerPool
// CheckerPool is implemented by the project system to provide checkers with
// request-scoped lifetime and reclamation. It returns a checker and a release
// function that must be called when the caller is done with the checker.
// The returned checker must not be accessed concurrently; each acquisition is exclusive.
// If file is non-nil, the pool may use it as an affinity hint to return the same
// checker for the same file across calls.
// PORT: `file` is `Node::NIL` for Go nil. The caller borrows the checker
// (`borrow_mut`) while it uses it.


// ---------------------------------------------------------------------------
// Program registry
// ---------------------------------------------------------------------------













// ---------------------------------------------------------------------------
// Current program
// ---------------------------------------------------------------------------












// ---------------------------------------------------------------------------
// Program construction and release
// ---------------------------------------------------------------------------

// Go: compiler/program.go:285 NewProgram
// PORT: Go `opts.CreateCheckerPool` is the `create_checker_pool` argument.
// The frontend program parses with no current program, then becomes a
// program version of the process (`program::new_program_version`).
// PORT: Go runs `initCheckerPool` before `verifyCompilerOptions`. Here the
// pool is set up after the program is built. Neither step reads the other.
// #4712: Go `collectContentMapperOptionDiagnostics` runs when the program
// version is built (`go_frontend::content_mapper_option_diagnostics_of`).

// Go: compiler/program.go:305 UpdateProgram
// PORT: `NewProgram::update_program` builds the new program. It
// becomes a program version that shares the unchanged file versions of
// `p`, and gets its checker pool here. `create_checker_pool`, when set,
// overrides the one of `p` (Go `newOpts.CreateCheckerPool`).
// `create_module_resolver` goes to `NewProgram::update_program`; a Go nil
// `createModuleResolver` is `None` (ts#64299).




// Go: compiler/program.go:435 initCheckerPool
// PORT: `load_host` is not in Go (`ProgramCheckers::load_host`).

// Go: compiler/program.go:450 GetCheckerPool
// GetCheckerPool returns the checker pool associated with this program.

// Go: checker/checker.go:911 NewChecker (the checker id)
// PORT: Go `NewChecker(program, tracer)` also returns the checker mutex; a
// pool here holds `Rc<RefCell<Checker>>` and the borrow is the lock. The
// tracer is dropped. Go `program.BindSourceFiles()` runs inside
// `Checker::new` (`bind_all`), with the program of `p` current.
// PORT: Go `c.id = nextCheckerID.Add(1)`. `Checker::new(index)` sets
// `id = index + 1`, so the index is the counter value before the add.




// ---------------------------------------------------------------------------
// Compiler checker pool (Go compiler/checkerpool.go)
// ---------------------------------------------------------------------------

// Go: compiler/checkerpool.go:26 checkerPool
// PORT: the pool lives on the dispatch thread. Go `locks` are the
// `RefCell` of each checker: a caller holds `borrow_mut` where Go holds the
// lock, and a second borrow panics where Go would block. `tracing` is dropped.
// PORT: Go `checkers` starts as a list of nil pointers; each slot here is
// an empty `OnceCell` until `createCheckers`. Go `fileAssociations` maps a
// file to its checker; here it maps the file index to the checker index.

// ---------------------------------------------------------------------------
// Checker association (Go compiler/checkerpool.go, #4313)
// ---------------------------------------------------------------------------

// Go: compiler/checkerpool.go:38
//
// Checker association is a balanced graph-partitioning problem:
//
//   - A vertex is a source file.
//   - An undirected edge connects two files for each resolved, in-program import
//     entry between them. Multiple entries may connect the same pair and therefore
//     strengthen their affinity. Self-imports and unresolved or external targets do
//     not create edges.
//   - A partition is a checker with its own symbol, type, and instantiation caches.
//
// Putting related files on the same checker reduces duplicated cache construction,
// but concentrating too many roots on one checker increases the parallel critical
// path. We use weighted FENNEL to trade off those objectives:
//
//   affinity(partition) - alpha * incrementalLoadPenalty(partition)
//
// See Tsourakakis et al., "FENNEL: Streaming Graph Partitioning for Massive Scale
// Graphs", WSDM 2014: https://doi.org/10.1145/2556195.2556213.
//
// FENNEL is sensitive to stream order. Go's comment gives the calibration data;
// stream order is part of the policy below, rather than an incidental
// implementation detail.
//
// Go: compiler/checkerpool.go:74
// The constants below are empirical safety factors for a work proxy that cannot
// observe future semantic cache construction. They were swept across representative
// projects including VS Code, TypeScript, MUI docs, XState, and Bluesky, with 2, 4,
// and 8 checkers (Go's comment lists each sweep). These are project-independent
// operating points, not formulas derived by FENNEL.
// PORT: Go `int` is `i64` for weights and multipliers.

// Go: compiler/checkerpool.go:100 const(
pub const CHECKER_ASSOCIATION_TEXT_WEIGHT_DIVISOR: i64 = 100;
pub const CHECKER_ASSOCIATION_SOURCE_FILE_WEIGHT_MULTIPLIER: i64 = 4;
pub const CHECKER_ASSOCIATION_BALANCE_PENALTY_MULTIPLIER: i64 = 16;
pub const CHECKER_ASSOCIATION_PRIORITIZED_SOURCE_PENALTY: i64 = 12;
pub const CHECKER_ASSOCIATION_STRONG_BALANCE_MIN_CHECKER_COUNT: usize = 4;

// Go: compiler/checkerpool.go:108 checkerAssociationPolicy
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CheckerAssociationPolicy {
    pub prioritize_source_files: bool,
    pub source_file_weight_multiplier: i64,
    pub balance_penalty_multiplier: i64,
}

// Go: compiler/checkerpool.go:131 getCheckerAssociationPolicy
// getCheckerAssociationPolicy selects one of three calibrated regimes:
//
//  1. Source-dominated, any checker count:
//     source files first by descending weight; unmodified source-file weight;
//     checkerAssociationPrioritizedSourcePenalty.
//  2. Declaration-heavy, at least checkerAssociationStrongBalanceMinCheckerCount:
//     program order; checkerAssociationSourceFileWeightMultiplier;
//     checkerAssociationBalancePenaltyMultiplier.
//  3. Declaration-heavy, fewer checkers:
//     program order; unmodified source-file weight; unscaled adapted FENNEL
//     penalty.
//
// The source-dominated test is evaluated first intentionally: projects with very
// little declaration work benefit from balancing source-file roots directly even
// with a small checker pool.
#[must_use]
pub fn get_checker_association_policy(
    total_weight: i64,
    declaration_weight: i64,
    checker_count: usize,
) -> CheckerAssociationPolicy {
    if should_prioritize_source_files(total_weight, declaration_weight, checker_count) {
        return CheckerAssociationPolicy {
            prioritize_source_files: true,
            source_file_weight_multiplier: 1,
            balance_penalty_multiplier: CHECKER_ASSOCIATION_PRIORITIZED_SOURCE_PENALTY,
        };
    }
    if checker_count >= CHECKER_ASSOCIATION_STRONG_BALANCE_MIN_CHECKER_COUNT {
        return CheckerAssociationPolicy {
            prioritize_source_files: false,
            source_file_weight_multiplier: CHECKER_ASSOCIATION_SOURCE_FILE_WEIGHT_MULTIPLIER,
            balance_penalty_multiplier: CHECKER_ASSOCIATION_BALANCE_PENALTY_MULTIPLIER,
        };
    }
    CheckerAssociationPolicy {
        prioritize_source_files: false,
        source_file_weight_multiplier: 1,
        balance_penalty_multiplier: 1,
    }
}

// Go: compiler/checkerpool.go:164 getCheckerAssociationsInOrder
// getCheckerAssociationsInOrder partitions the import graph using a weighted adaptation
// of FENNEL's streaming graph-partitioning objective with gamma = 3/2. Each file
// is placed where it has the most already-placed neighbors, minus the incremental
// convex load penalty. The published alpha = m*sqrt(k)/n^(3/2) becomes
// m*sqrt(k)/W^(3/2), where W is total estimated checker work. penaltyMultiplier
// applies the empirical safety factor selected by getCheckerAssociationPolicy.
//
// A nil order means stable program order. The preferred maximum checker weight is
// the larger of the largest file and roughly 101% of average. If no checker can
// accept a file under that bound, the file is assigned to the least-loaded checker.
// The 1% slack permits discrete files to pack near the average while preventing
// affinity from deliberately creating meaningful estimated imbalance. Ties are
// deterministic.
// PORT: Go nil `fileOrder` is `None`. Go returns nil for no file; this
// returns an empty list. The float math is in Go's order, so the scores are
// the same bits.

// Go: compiler/checkerpool.go:239 getCheckerAssociationOrder
// getCheckerAssociationOrder places source files before declarations and
// orders each group by descending estimated work. This exposes expensive semantic
// roots early, when all checker loads are still available. Returning nil preserves
// program order without allocating an index array. Program order is itself a
// locality choice: it preserves deterministic groups produced during program
// construction and was consistently safer for declaration-heavy projects.
// PORT: Go nil is `None`.

// Go: compiler/checkerpool.go:261 getCheckerAssociationBaseWeight
#[must_use]
pub fn get_checker_association_base_weight(node_count: i64, text_length: i64) -> i64 {
    (node_count + text_length / CHECKER_ASSOCIATION_TEXT_WEIGHT_DIVISOR).max(1)
}

// Go: compiler/checkerpool.go:274 shouldPrioritizeSourceFiles
// shouldPrioritizeSourceFiles reports whether all declaration-file base work is at
// most half of one average checker load:
//
//	declarationWeight <= totalWeight / (2 * checkerCount)
//
// This threshold separated source-dominated projects such as VS Code from projects
// where declaration locality remained important, such as MUI docs, TypeScript, and
// XState. Delaying at most half a checker-load of declarations was the stable
// boundary in the cross-project sweeps.
#[must_use]
pub fn should_prioritize_source_files(
    total_weight: i64,
    declaration_weight: i64,
    checker_count: usize,
) -> bool {
    declaration_weight * checker_count as i64 * 2 <= total_weight
}

// Go: compiler/checkerpool.go:285 getCheckerAssociationWeights
// getCheckerAssociationWeights combines local syntax work with syntactic import
// fanout. One import unit is totalBaseWeight / totalImports, so imports collectively
// contribute approximately the same vertex weight as syntax. Syntactic imports are
// deliberately broader than getImportAdjacency's resolved, in-program edges: this
// term estimates the work of processing module references, while adjacency controls
// checker affinity. Normalizing the term avoids a project-specific vertex-weight
// constant.
#[must_use]
pub fn get_checker_association_weights(base_weights: &[i64], import_counts: &[i64]) -> Vec<i64> {
    let mut total_base_weight: i64 = 0;
    let mut total_imports: i64 = 0;
    for (i, &base_weight) in base_weights.iter().enumerate() {
        total_base_weight += base_weight;
        total_imports += import_counts[i];
    }
    let mut import_weight: i64 = 0;
    if total_imports > 0 {
        import_weight = (total_base_weight / total_imports).max(1);
    }
    base_weights
        .iter()
        .zip(import_counts)
        .map(|(&base_weight, &import_count)| base_weight + import_count * import_weight)
        .collect()
}


// Go: compiler/checkerpool.go:421 (*checkerPool).getImportAdjacency
// getImportAdjacency returns an undirected import graph represented by file
// index. A directed import from A to B makes both files adjacent because either
// file can benefit from sharing checker caches with the other.
// PORT: Go ranges over the resolution map in random order; the order of a
// list does not change the associations, only its length and contents do.

// Go: compiler/checkerpool.go:303 newCheckerPool

// Go: compiler/checkerpool.go:307 newCheckerPoolWithTracing
// PORT: tracing is dropped.



// ---------------------------------------------------------------------------
// Program checker access (Go compiler/program.go:445-492)
// ---------------------------------------------------------------------------

// Go: compiler/program.go:574 BindSourceFiles
// PORT: the Rust binder binds every file of the program version into one
// arena (`program::bind_all`). A file version that an earlier version
// bound is not bound again.

// PORT: each `get_type_checker*` function makes the program of `p` current
// until the returned release runs (see the module comment), so the caller
// uses the checker with its program current.

// Go: compiler/program.go:590 GetTypeChecker
// Return the type checker associated with the program.

// Go: compiler/program.go:597 ForEachCheckerParallel

// Go: compiler/program.go:607 GetTypeCheckerForFile
// Return a checker for the given file. We may have multiple checkers in concurrent scenarios and this
// method returns the checker that was tasked with checking the file. Note that it isn't possible to mix
// types obtained from different checkers, so only non-type data (such as diagnostics or string
// representations of types) should be obtained from checkers returned by this method.

// Go: compiler/program.go:616 GetTypeCheckerForFileExclusive
// Return a checker for the given file, locked to the current thread to prevent data races from multiple threads
// accessing the same checker. The lock will be released when the `done` function is called.

// ---------------------------------------------------------------------------
// Diagnostics (Go compiler/program.go:534-712 and 1290-1420)
// ---------------------------------------------------------------------------
//
// PORT: the per-file bodies that already exist in program.rs for the
// current program are called from here: `get_semantic_diagnostics_with_checker`
// (Go program.go:1315 getSemanticDiagnosticsWithChecker),
// `get_bind_and_check_diagnostics_with_checker` (:1325),
// `get_diagnostics_with_preceding_directives` (:1359),
// `get_suggestion_diagnostics_with_checker` (:1410) and
// `get_additional_js_syntactic_diagnostics` (:618). They read the file data
// of the current program; each public function here makes `p` current.


// Go: compiler/program.go:667 collectDiagnostics
// collectDiagnostics collects diagnostics from a single file or all files.
// If sourceFile is non-nil, returns diagnostics for just that file.
// If sourceFile is nil, returns diagnostics for all files in the program.

// Go: compiler/program.go:678 collectDiagnosticsFromFiles
// PORT: Go runs the files on a WorkGroup. On the dispatch thread they run
// one after another, in file order.

// Go: compiler/program.go:695 collectCheckerDiagnostics
// collectCheckerDiagnostics collects diagnostics from a single file or all files,
// using a callback that receives the checker for each file. When the checker pool
// supports grouped iteration (compiler pool), files are grouped by checker and
// processed in parallel with one task per checker, reducing contention and improving
// cache locality. Otherwise, falls back to per-file concurrent collection.

// Go: compiler/program.go:720 collectCheckerDiagnosticsFromFiles
// collectCheckerDiagnosticsFromFiles collects checker diagnostics for a list of files.
// PORT: Go runs the files of an external pool on a WorkGroup. On the
// dispatch thread they run one after another, in file order.

// Go: compiler/program.go:743 GetSyntacticDiagnostics

// Go: compiler/program.go:787 GetBindDiagnostics
// PORT: Go binds only `sourceFile` when it is set, else every file. The
// Rust binder binds every file of the program version into one arena, so
// both cases bind all files.

// Go: compiler/program.go:798 GetSemanticDiagnostics

// Go: compiler/program.go:804 GetSemanticDiagnosticsForIncremental
// GetSemanticDiagnosticsForIncremental includes newly discovered globals in each
// file's cached diagnostics and leaves noEmit filtering to the builder.

// Go: compiler/program.go:815 GetSuggestionDiagnostics

// Go: compiler/program.go:819 GetProgramDiagnostics

// Go: compiler/program.go:840 GetIncludeProcessorDiagnostics

// Go: compiler/program.go:848 SkipTypeChecking

// Go: compiler/program.go:856 canIncludeBindAndCheckDiagnostics

// Go: compiler/program.go:1455 GetGlobalDiagnostics

// Go: compiler/program.go:1467 GetDeclarationDiagnostics

// Go: compiler/program.go:1618 getDeclarationDiagnosticsForFile

// Go: compiler/emitHost.go:38 newEmitHost
// PORT: the language-service form of `program::new_emit_host`. The checker
// comes from `GetTypeCheckerForFile` of `p`, which a dispatch-thread pool
// shares as `Rc<RefCell<Checker>>`, so the resolver links to that `Rc`
// (`get_emit_resolver_of_shared_checker`) instead of a compile worker
// checker. The host methods read the current program, which is `p`.

// Go: compiler/program.go:1867 Program.Emit
// PORT: the language service form of `program_emit::emit` (#4710: the
// language server flake check; the API emit methods call Go `Program.Emit`
// too). Each file's emit host gets its checker from the checker pool of `p`
// (`new_emit_host`), as in Go, so the emit uses the checkers that the
// diagnostics use, not a compile worker pool. Go runs the files on a
// WorkGroup. On the dispatch thread they run one after another, in file
// order. A language service program has no Go `opts.Tracing`, so there is
// no trace event. Go returns nil when `ctx` is done after
// `HandleNoEmitOptions` gave nil; here that is `EmitResult::default()`.

// Go: compiler/program.go:1976 HandleNoEmitOptions
// PORT: the language service form of `program_emit::handle_no_emit_options`
// (`Program.Emit` passes a nil `emitBuildInfo`). The bind, semantic, global
// and declaration diagnostics come from `p` and its checker pool. The
// config, syntactic and program diagnostics of `get_diagnostics_of_any_program`
// read the current program, which is `p`.

// Go: compiler/program.go:1785 IsGlobalTypingsFile
