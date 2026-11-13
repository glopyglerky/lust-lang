//! Port of Effect-TS/tsgo `internal/rules/effect_map_flatten.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



#[derive(Clone, Debug)]
pub struct EffectMapFlattenMatch {
    pub source_file: Node,
    pub location: TextRange,
    pub node: Node,
}

