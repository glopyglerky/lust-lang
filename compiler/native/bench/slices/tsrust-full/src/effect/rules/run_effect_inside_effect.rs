// Go: internal/rules/run_effect_inside_effect.go

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

/// runEffectApis lists the Effect module run APIs that should not be called inside Effect generators.
static RUN_EFFECT_APIS: &[&str] = &["runSync", "runPromise", "runFork", "runCallback"];



/// RunEffectInsideEffectMatch holds the diagnostic and AST nodes needed by both the
/// diagnostic rule and the quick-fix for the runEffectInsideEffect pattern.
#[derive(Clone, Debug, Default)]
pub struct RunEffectInsideEffectMatch {
    /// The source file where this match was found
    pub source_file: Node,
    /// Pre-computed error range (on the callee expression)
    pub location: TextRange,
    /// The full call expression node (e.g., Effect.runPromise(check))
    pub call_node: Node,
    /// The callee expression (e.g., Effect.runPromise)
    pub callee_node: Node,
    /// The matched run API name (e.g., "runPromise")
    pub method_name: String,
    /// True when call is in a nested scope rather than direct generator scope
    pub is_nested_scope: bool,
    /// The enclosing Effect generator function
    pub generator_function: Node,
}

// Go: rules/run_effect_inside_effect.go AnalyzeRunEffectInsideEffect

// Go: rules/run_effect_inside_effect.go analyzeRunEffectInsideEffectNode

// Go: rules/run_effect_inside_effect.go matchRunEffectApi
