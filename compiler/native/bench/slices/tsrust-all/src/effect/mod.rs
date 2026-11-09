//! Native Effect diagnostics: a port of Effect-TS/tsgo at
//! `@effect/tsgo@0.46.1` (`f1a7cad0292d9d315e7f87694f127f605711d55e`).
//!
//! The reference patches typescript-go in a few places (`_patches/typescript`).
//! Each patch site here names its patch. The rules run after the checker has
//! checked a source file, with the same checker, and add their diagnostics
//! to the checker's diagnostics (codes 377000 to 377999). Nothing runs unless
//! the tsconfig has an `@effect/language-service` plugin entry with
//! diagnostics enabled. A standalone API process (`tsgo --api`) runs nothing
//! unless `TSGO_EFFECT_API=1` (`rulerunner::set_api_process`).
//!
//! Read `crates/ts_goport/src/effect/PORTING.md` before editing.

use crate::prelude::*;

pub mod configcheck;
pub mod configraw;
pub mod diag;
pub mod directives;
pub mod etscore;
pub mod graph;
pub mod keybuilder;
pub mod layergraph;
pub mod pluginoptions;
pub mod rule;
pub mod rulerunner;
pub mod rules;
pub mod typeparser;

/// Effect-TS/tsgo patch 005 `RelationError`: a type relation error that the
/// relater reported, kept for Effect rules.
#[derive(Clone, Copy, Debug)]
pub struct RelationError {
    pub source: TypeId,
    pub target: TypeId,
    pub error_node: Node,
}

/// Go `rule.IsEffectCode`.
#[must_use]
pub fn is_effect_code(code: i32) -> bool {
    (377_000..378_000).contains(&code)
}

// Go: etscheckerhooks/init.go afterCheckSourceFile

// Go: checker.go (patch 002) GetRelationErrors

// Go: etsexecutehooks/init.go FilterDiagnosticsForExitCode

