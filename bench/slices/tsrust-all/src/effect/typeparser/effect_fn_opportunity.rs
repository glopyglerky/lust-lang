//! Port of Effect-TS/tsgo `internal/typeparser/effect_fn_opportunity.go`.

use crate::effect::typeparser::*;
use crate::prelude::*;

/// EffectFnOpportunityResult represents a function that can be converted to Effect.fn.
// Go: typeparser/effect_fn_opportunity.go EffectFnOpportunityResult
#[derive(Clone)]
pub struct EffectFnOpportunityResult {
    /// The function node being reported
    pub target_node: Node,
    /// The discovered name node for the function
    pub name_identifier: Node,
    /// Non-nil for gen opportunity, nil for regular
    pub generator_function: Node,
    /// Pipe args from piped Effect.gen (may be empty)
    pub pipe_arguments: Vec<Node>,
    /// Span name from Effect.withSpan if last pipe arg, or nil
    pub explicit_trace_expression: Node,
    /// The local function/variable name for suggested span
    pub suggested_trace_name: String,
    /// Context-aware name (e.g., "ServiceTag.member" or exported name)
    pub inferred_trace_name: String,
    /// The Effect module identifier
    pub effect_module: Node,
    /// True for gen opportunity, false for regular
    pub has_gen_body: bool,
    /// True when the target is a property value inside a Layer service definition
    pub is_layer_member: bool,
}



/// genOpportunityResult holds the parsed gen opportunity data.
// Go: typeparser/effect_fn_opportunity.go genOpportunityResult
#[derive(Clone)]
pub struct GenOpportunityResult {
    pub effect_module: Node,
    pub generator_function: Node,
    pub pipe_arguments: Vec<Node>,
    pub explicit_trace_expression: Node,
}










