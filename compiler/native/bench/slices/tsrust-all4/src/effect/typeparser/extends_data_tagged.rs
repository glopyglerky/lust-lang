// Go: internal/typeparser/extends_data_tagged.go

use crate::effect::typeparser::*;
use crate::prelude::*;

/// DataTaggedErrorResult holds the parsed result of a class extending Data.TaggedError.
#[derive(Clone, Debug)]
pub struct DataTaggedErrorResult {
    /// The class name identifier
    pub class_name: Node,
    /// The key string literal from the call's first argument, or nil
    pub key_string_literal: Node,
}

