//! Port of Effect-TS/tsgo `internal/rules/unnecessary_effect_gen.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

// Go: rules.UnnecessaryEffectGen


// Go: rules.UnnecessaryEffectGenMatch
/// UnnecessaryEffectGenMatch holds the AST nodes needed by both the diagnostic rule
/// and the quick-fix for the unnecessaryEffectGen pattern.
#[derive(Clone)]
pub struct UnnecessaryEffectGenMatch {
    /// The source file where this match was found
    pub source_file: Node,
    /// The pre-computed error range for this match
    pub location: TextRange,
    /// The Effect.gen(...) call expression (replacement target)
    pub call_node: Node,
    /// The expression being yield*-ed (the replacement value)
    pub yielded_expression: Node,
    /// Whether the statement had an explicit return
    pub explicit_return: bool,
    /// Whether the yielded Effect's success type is void-like
    pub success_is_void: bool,
    /// The Effect module identifier (for Effect.asVoid wrapping)
    pub effect_module_node: Node,
}

// Go: rules.AnalyzeUnnecessaryEffectGen


// Go: rules.analyzeUnnecessaryEffectGenNode

// Go: rules.isYieldStarExpression
