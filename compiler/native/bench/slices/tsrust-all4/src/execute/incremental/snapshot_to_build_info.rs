//! Port of execute/incremental/snapshottobuildinfo.go.
//!
//! PORT: Go `*compiler.Program` is the current program (`prog()`). Go nil
//! slices in `BuildInfo` are `None`: a Go `append` to a nil slice or a
//! `core.Map` of a non-empty slice makes `Some`, and nothing appended (or a
//! `core.Map` of a nil or empty `slices.Collect`) stays `None`.

use super::build_info::*;
use super::checker_access::*;
use super::hash::FileInfo;
use super::hash::*;
use super::snapshot::*;
use crate::frontend::prelude::*;
use crate::gostd::GoError;
use crate::gostd::slices::stable_sort_by;

/// Go `core.Map` over a slice that is nil when empty (`slices.Collect`,
/// diagnostic chains, message args).
fn non_empty<T>(items: Vec<T>) -> Option<Vec<T>> {
    if items.is_empty() { None } else { Some(items) }
}

// Go: incremental/snapshottobuildinfo.go:18 snapshotToBuildInfo
// PORT: Go `program.ContentMapperProject()` is the Go frontend program's
// (none when no program is loaded).

// Go: incremental/snapshottobuildinfo.go:64 toBuildInfo
// PORT: Go `roots map[*ast.SourceFile]tspath.Path` is an `FxIndexMap`; Go
// sorts its keys before use.


// Go: incremental/snapshottobuildinfo.go:181 toBuildInfoRepopulateInfo
#[must_use]
pub fn to_build_info_repopulate_info(
    info: Option<&RepopulateDiagnosticInfo>,
) -> Option<BuildInfoRepopulateInfo> {
    let info = info?;
    Some(BuildInfoRepopulateInfo {
        kind: info.kind,
        module_reference: info.module_reference.clone(),
        mode: info.mode,
        package_name: info.package_name.clone(),
    })
}

