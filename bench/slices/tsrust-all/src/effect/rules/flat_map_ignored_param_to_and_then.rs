//! Port of Effect-TS/tsgo `internal/rules/flat_map_ignored_param_to_and_then.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



/// FlatMapIgnoredParamToAndThenMatch holds the nodes needed by the diagnostic
/// and quick fix.
// Go: rules/flat_map_ignored_param_to_and_then.go FlatMapIgnoredParamToAndThenMatch
#[derive(Clone, Debug)]
pub struct FlatMapIgnoredParamToAndThenMatch {
    pub source_file: Node,
    pub location: TextRange,
    pub callee: Node,
    pub callee_name_node: Node,
    pub callback: Node,
    pub effect_value: Node,
}

