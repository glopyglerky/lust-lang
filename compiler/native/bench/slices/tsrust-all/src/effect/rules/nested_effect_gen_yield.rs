//! Port of Effect-TS/tsgo `internal/rules/nested_effect_gen_yield.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



// Go: rules/nested_effect_gen_yield.go NestedEffectGenYieldMatch
#[derive(Clone, Copy)]
pub struct NestedEffectGenYieldMatch {
    pub location: TextRange,
}

// Go: rules/nested_effect_gen_yield.go AnalyzeNestedEffectGenYield
