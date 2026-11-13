//! Port of Effect-TS/tsgo `internal/typeparser/effect_fn.go`.

use crate::effect::typeparser::*;
use crate::prelude::*;

// PORT: Go `type EffectFnVariant string`. `as_str` gives the Go string.
// Go: typeparser/effect_fn.go EffectFnVariant
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EffectFnVariant {
    Fn,
    FnUntraced,
    FnUntracedEager,
}

impl EffectFnVariant {
    /// The Go string value of the variant.
    pub const fn as_str(self) -> &'static str {
        match self {
            EffectFnVariant::Fn => "fn",
            EffectFnVariant::FnUntraced => "fnUntraced",
            EffectFnVariant::FnUntracedEager => "fnUntracedEager",
        }
    }
}

impl std::fmt::Display for EffectFnVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// EffectFnCallResult represents a parsed Effect.fn-family call.
// Go: typeparser/effect_fn.go EffectFnCallResult
#[derive(Clone)]
pub struct EffectFnCallResult {
    pub call: Node,
    pub variant: EffectFnVariant,
    pub effect_module: Node,
    pub options_node: Node,
    /// ArrowFunction or FunctionExpression
    pub function_node: Node,
    pub function_return_type: TypeId,
    /// Transformation args after the body (may be empty/nil)
    pub pipe_arguments: Vec<Node>,
    pub pipe_args_out_type: Vec<TypeId>,
    /// The name string from curried Effect.fn("name")(...), or nil
    pub trace_expression: Node,
}

impl EffectFnCallResult {
    // Go: typeparser/effect_fn.go EffectFnCallResult.IsGenerator

    // Go: typeparser/effect_fn.go EffectFnCallResult.GeneratorFunction

    // Go: typeparser/effect_fn.go EffectFnCallResult.Body
}

// Go: typeparser/effect_fn.go splitEffectFnArguments

// Go: typeparser/effect_fn.go isGeneratorFunctionNode

