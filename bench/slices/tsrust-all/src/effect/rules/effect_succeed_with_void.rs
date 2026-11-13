//! Port of Effect-TS/tsgo `internal/rules/effect_succeed_with_void.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



/// EffectSucceedWithVoidMatch holds the AST nodes needed by both the diagnostic rule
/// and the quick-fix for the effectSucceedWithVoid pattern.
#[derive(Clone, Debug)]
pub struct EffectSucceedWithVoidMatch {
    /// The source file where the diagnostic should be reported
    pub source_file: Node,
    /// The pre-computed error range for this match
    pub location: TextRange,
    /// The Effect.succeed(...) call expression (replacement target)
    pub call_node: Node,
    /// The Effect module identifier (e.g., "Effect" in Effect.succeed)
    pub effect_module_node: Node,
}


