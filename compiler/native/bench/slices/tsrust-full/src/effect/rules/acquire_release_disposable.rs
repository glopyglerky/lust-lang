//! Port of Effect-TS/tsgo `internal/rules/acquire_release_disposable.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

use super::all_of_map_to_for_each::{};

// Go: rules.AcquireReleaseDisposable


// Go: rules.AcquireReleaseDisposableMatch
/// AcquireReleaseDisposableMatch holds the nodes needed by the diagnostic and
/// its quick fix.
#[derive(Clone)]
pub struct AcquireReleaseDisposableMatch {
    pub source_file: Node,
    pub location: TextRange,
    pub call_node: Node,
    pub effect_module: Node,
    pub acquire: Node,
    pub has_type_arguments: bool,
}

// Go: rules.AnalyzeAcquireReleaseDisposable


// Go: rules.globalDisposableUnion

// Go: rules.analyzeAcquireReleaseDisposableCall

// Go: rules.isDefinitelyDisposable

// Go: rules.isDisposalRelease

// Go: rules.isPlainIdentifierParameter

// Go: rules.disposalProtocolCall
