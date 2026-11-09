//! Port of execute/incremental/buildinfotosnapshot.go.

use super::build_info::*;
use super::hash::FileInfo;
use super::hash::*;
use super::snapshot::*;
use crate::frontend::prelude::*;
use std::sync::Arc;

// Go: incremental/buildinfotosnapshot.go:12 buildInfoToSnapshot
// PORT: the options read from the build info are leaked to get the
// `&'static CompilerOptions` that `Snapshot` keeps (see `Snapshot`). A
// process reads at most one build info program.

// Go: incremental/buildinfotosnapshot.go:49 toSnapshot


/// Go `s[i]`. An id of a bad `.tsbuildinfo` can be out of range, and Go
/// panics with the runtime text.
#[track_caller]
fn go_index<T>(s: &[T], i: i64) -> &T {
    match usize::try_from(i).ok().and_then(|at| s.get(at)) {
        Some(item) => item,
        None if i < 0 => go_panic(format!("runtime error: index out of range [{i}]")),
        None => go_panic(format!(
            "runtime error: index out of range [{i}] with length {}",
            s.len()
        )),
    }
}

// Go: incremental/buildinfotosnapshot.go:102 fromBuildInfoRepopulateInfo
#[must_use]
pub fn from_build_info_repopulate_info(
    info: Option<&BuildInfoRepopulateInfo>,
) -> Option<RepopulateInfoRef> {
    let info = info?;
    Some(RepopulateInfoRef::new(RepopulateDiagnosticInfo {
        kind: info.kind,
        module_reference: info.module_reference.clone(),
        mode: info.mode,
        package_name: info.package_name.clone(),
    }))
}
