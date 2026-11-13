// Go: internal/typeparser/extends_schema_opaque.go

use crate::effect::typeparser::*;
use crate::prelude::*;

/// SchemaOpaqueResult holds the type argument from a class extending Schema.Opaque.
#[derive(Clone, Debug)]
pub struct SchemaOpaqueResult {
    pub self_type_node: Node,
}

