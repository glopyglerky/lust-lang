//! Port of Effect-TS/tsgo `internal/pluginoptions`: the per-file plugin
//! options after the `overrides` whose globs match the file.

use crate::effect::etscore::{
    Severity,
};
use crate::frontend::vfs::vfsmatch::{Usage, };
use crate::indexmap::IndexMap;

// Go: pluginoptions ResolveEffectPluginOptionsForSourceFile

// Go: pluginoptions ResolveDiagnosticSeverityForFile



