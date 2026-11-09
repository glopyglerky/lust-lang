// Go: internal/typeparser/extends_sql_model_class.go

use crate::effect::typeparser::*;
use crate::prelude::*;

/// SqlModelClassResult holds the parsed result of a class extending Model.Class.
#[derive(Clone, Debug)]
pub struct SqlModelClassResult {
    /// The class name identifier
    pub class_name: Node,
    /// The Self type argument node (first type arg of the inner call)
    pub self_type_node: Node,
}

