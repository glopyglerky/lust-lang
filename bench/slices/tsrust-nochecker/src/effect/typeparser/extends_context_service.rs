// Go: internal/typeparser/extends_context_service.go

use crate::effect::typeparser::*;
use crate::prelude::*;

/// ServiceMapServiceResult holds the parsed result of a class extending Context.Service.
#[derive(Clone, Debug)]
pub struct ServiceMapServiceResult {
    /// The class name identifier
    pub class_name: Node,
    /// The Self type argument node (first type arg of the inner call)
    pub self_type_node: Node,
    /// The key string literal from the outer call's first argument, or nil
    pub key_string_literal: Node,
}

