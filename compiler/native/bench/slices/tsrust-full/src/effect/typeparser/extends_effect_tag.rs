// Go: internal/typeparser/extends_effect_tag.go

use crate::effect::typeparser::*;
use crate::prelude::*;

/// EffectTagResult holds the parsed result of a class extending Effect.Tag.
#[derive(Clone, Debug)]
pub struct EffectTagResult {
    /// The class name identifier
    pub class_name: Node,
    /// The Self type argument node (first type arg of the outer call)
    pub self_type_node: Node,
    /// The key string literal from the inner call's first argument, or nil
    pub key_string_literal: Node,
}

