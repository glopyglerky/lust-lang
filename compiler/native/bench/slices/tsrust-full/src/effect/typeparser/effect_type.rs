// Go: internal/typeparser/effect_type.go

use crate::effect::typeparser::*;
use crate::prelude::*;
use std::sync::LazyLock;



/// EffectTypeId is the property key for Effect's variance struct.
/// Effect v4 (effect-smol) uses this pattern to encode type parameters.
pub const EFFECT_TYPE_ID: &str = "~effect/Effect";



// Go: isEffectTypeSourceFile

