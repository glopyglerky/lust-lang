//! Port of Effect-TS/tsgo `internal/rules/match_effect_to_map_both.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



pub struct MatchEffectToMapBothMatch {
    pub source_file: Node,
    pub location: TextRange,
    pub callee_node: Node,
    pub callee_name_node: Node,
    pub handler_results: [Node; 2],
    pub constructor_args: [Node; 2],
}

// Go: rules/match_effect_to_map_both.go AnalyzeMatchEffectToMapBoth
