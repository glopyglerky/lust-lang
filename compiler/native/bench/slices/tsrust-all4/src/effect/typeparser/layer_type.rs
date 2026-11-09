// Go: internal/typeparser/layer_type.go

use crate::effect::typeparser::*;
use crate::prelude::*;
use std::cmp::Ordering;
use std::sync::LazyLock;


/// LayerTypeId is the property key for Layer's variance struct.
pub const LAYER_TYPE_ID: &str = "~effect/Layer";

/// Layer represents parsed Layer<ROut, E, RIn> type parameters.
#[derive(Clone, Debug)]
pub struct Layer {
    /// Provided services (contravariant)
    pub r_out: TypeId,
    /// Error type (covariant)
    pub e: TypeId,
    /// Required services (covariant)
    pub r_in: TypeId,
}


// Go: typeparser/layer_type.go isLayerTypeSourceFile

