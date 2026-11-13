// Go: internal/typeparser/extends_schema_class.go

use crate::effect::typeparser::*;
use crate::prelude::*;

/// SchemaClassResult holds the parsed result of a class extending Schema.Class or Schema.RequestClass.
#[derive(Clone, Debug)]
pub struct SchemaClassResult {
    /// The class name identifier
    pub class_name: Node,
    /// The Self type argument node (first type arg of the inner call)
    pub self_type_node: Node,
}

