// Go: internal/typeparser/extends_effect_model_class.go

use crate::effect::typeparser::*;
use crate::prelude::*;

/// EffectModelClassResult holds the parsed result of a class extending Model.Class
/// from effect/unstable/schema.
#[derive(Clone, Debug)]
pub struct EffectModelClassResult {
    /// The class name identifier
    pub class_name: Node,
    /// The Self type argument node (first type arg of the inner call)
    pub self_type_node: Node,
}

