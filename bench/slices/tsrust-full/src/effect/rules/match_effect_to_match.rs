// Go: internal/rules/match_effect_to_match.go

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



#[derive(Clone, Debug)]
pub struct MatchEffectToMatchMatch {
    pub source_file: Node,
    pub location: TextRange,
    pub callee_node: Node,
    pub callee_name_node: Node,
    pub match_effect_name: String,
    pub replacement_name: String,
    pub handler_results: [Node; 2],
    pub succeed_arguments: [Node; 2],
}

// Go: rules/match_effect_to_match.go AnalyzeMatchEffectToMatch

// Go: rules/match_effect_to_match.go matchEffectConstructorHandler
