//! Port of Effect-TS/tsgo `internal/rules/effect_fn_iife.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

// EffectFnIife detects Effect.fn or Effect.fnUntraced calls that are immediately invoked (IIFE pattern).


// EffectFnIifeMatch holds the parsed result needed by both the diagnostic rule
// and the quick-fix for the effectFnIife pattern.
#[derive(Clone, Debug)]
pub struct EffectFnIifeMatch {
    pub source_file: Node,
    pub location: TextRange,
    pub result: Rc<EffectFnIifeResult>,
}

#[derive(Clone, Debug)]
pub struct EffectFnIifeResult {
    pub outer_call: Node,
    pub inner_call: Node,
    pub effect_module: Node,
    pub variant: String,
    pub generator_function: Node,
    pub pipe_arguments: Vec<Node>,
    pub trace_expression: Node,
}

// AnalyzeEffectFnIife finds all Effect.fn or Effect.fnUntraced calls that are
// immediately invoked (IIFE pattern) in the given source file.

