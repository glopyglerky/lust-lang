// Go: internal/typeparser/extends_context_tag.go

use crate::effect::typeparser::*;
use crate::prelude::*;

/// ContextTagResult holds the parsed result of a class extending Context.Tag.
#[derive(Clone, Debug)]
pub struct ContextTagResult {
    /// The class name identifier
    pub class_name: Node,
    /// The Self type argument node (first type arg of the outer call)
    pub self_type_node: Node,
    /// The key string literal from the inner call's first argument, or nil
    pub key_string_literal: Node,
}

