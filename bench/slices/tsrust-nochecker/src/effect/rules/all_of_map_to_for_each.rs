//! Port of Effect-TS/tsgo `internal/rules/all_of_map_to_for_each.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

// Go: rules.AllOfMapToForEach


// Go: rules.AllOfMapToForEachMatch
/// AllOfMapToForEachMatch holds the nodes needed by the diagnostic and its fix.
#[derive(Clone)]
pub struct AllOfMapToForEachMatch {
    pub source_file: Node,
    pub location: TextRange,
    pub call_node: Node,
    pub effect_module: Node,
    pub receiver: Node,
    pub callback: Node,
    pub options: Node,
    pub has_type_arguments: bool,
    /// CanFix reports whether the match is a standalone Effect.all call the
    /// quick fix can rewrite. Data-last Effect.all references inside piping
    /// flows are diagnostic-only, since the fix would have to restructure the
    /// surrounding pipe.
    pub can_fix: bool,
}

// Go: rules.AnalyzeAllOfMapToForEach


// Go: rules.transformationApplicationNode

// Go: rules.analyzeAllOfMapToForEachCall

// Go: rules.analyzeAllOfMapToForEachReceiver

// Go: rules.containsSpreadElement

// Go: rules.hasCallTypeArguments
