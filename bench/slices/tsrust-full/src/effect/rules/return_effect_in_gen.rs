//! Port of Effect-TS/tsgo `internal/rules/return_effect_in_gen.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



/// ReturnEffectInGenMatch holds the diagnostic and the return statement node needed
/// by both the diagnostic rule and the quick-fix.
// Go: rules/return_effect_in_gen.go ReturnEffectInGenMatch
#[derive(Clone, Debug)]
pub struct ReturnEffectInGenMatch {
    /// The source file where this match was found
    pub source_file: Node,
    /// The pre-computed error range for this match
    pub location: TextRange,
    /// The return statement AST node
    pub return_node: Node,
}


