// Go: internal/typeparser/extends_schema_tagged.go

use crate::effect::typeparser::*;
use crate::prelude::*;

/// SchemaTaggedResult holds the parsed result of a class extending Schema.TaggedClass/TaggedError/TaggedRequest.
#[derive(Clone, Debug)]
pub struct SchemaTaggedResult {
    /// The class name identifier
    pub class_name: Node,
    /// The Self type argument node (first type arg of the inner call)
    pub self_type_node: Node,
    /// The identifier arg from the inner call (first arg), or nil
    pub key_string_literal: Node,
    /// The tag arg from the outer call (first arg), or nil
    pub tag_string_literal: Node,
}

