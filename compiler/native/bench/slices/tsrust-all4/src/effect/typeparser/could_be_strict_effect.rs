// Go: internal/typeparser/could_be_strict_effect.go

use crate::effect::typeparser::*;
use crate::prelude::*;
use std::sync::LazyLock;

/// strictEffectTypeNames are the type symbol names that StrictEffectType can
/// match. couldBeNamed takes a set so future prefilters for other wrapper
/// types (Stream, Layer, ...) can reuse the same conservative walk.
pub static STRICT_EFFECT_TYPE_NAMES: LazyLock<FxHashMap<&'static str, bool>> =
    LazyLock::new(|| FxHashMap::from_iter([("Effect", true)]));


