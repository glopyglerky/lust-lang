// Go: internal/typeparser/extends_effect_service.go

use crate::effect::typeparser::*;
use crate::prelude::*;

/// EffectServiceResult holds the parsed result of a class extending Effect.Service.
#[derive(Clone, Debug)]
pub struct EffectServiceResult {
    /// The class name identifier
    pub class_name: Node,
    /// The Self type argument node (first type arg of the inner call)
    pub self_type_node: Node,
    /// The options expression (second argument of the outer call), or nil
    pub options: Node,
    /// The key string literal from the outer call's first argument, or nil
    pub key_string_literal: Node,
}

