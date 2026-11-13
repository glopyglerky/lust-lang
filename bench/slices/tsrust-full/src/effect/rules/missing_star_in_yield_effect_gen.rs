//! Port of Effect-TS/tsgo `internal/rules/missing_star_in_yield_effect_gen.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



/// MissingStarInYieldEffectGenMatch holds the AST node needed by both the diagnostic rule
/// and the quick-fix for the missingStarInYieldEffectGen pattern.
// Go: rules/missing_star_in_yield_effect_gen.go MissingStarInYieldEffectGenMatch
#[derive(Clone, Debug)]
pub struct MissingStarInYieldEffectGenMatch {
    /// The source file where the diagnostic should be reported
    pub source_file: Node,
    /// The pre-computed error range for this match
    pub location: TextRange,
    /// The yield expression node (for fix range)
    pub yield_node: Node,
    /// The generator function node (for related info)
    pub gen_fn_node: Node,
}

