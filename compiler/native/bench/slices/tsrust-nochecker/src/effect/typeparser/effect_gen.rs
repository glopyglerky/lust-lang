//! Port of Effect-TS/tsgo `internal/typeparser/effect_gen.go`.

use crate::effect::typeparser::*;
use crate::prelude::*;

/// EffectGenCallResult represents a parsed Effect.gen(...) call.
// Go: typeparser/effect_gen.go EffectGenCallResult
#[derive(Clone)]
pub struct EffectGenCallResult {
    pub call: Node,
    /// Namespace receiver for Effect.gen; nil for named imports and local aliases.
    pub effect_module: Node,
    pub options_node: Node,
    pub generator_function: Node,
    pub body: Node,
    pub pipe_arguments: Vec<Node>,
}

