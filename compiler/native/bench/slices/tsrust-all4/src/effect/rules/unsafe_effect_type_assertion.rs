// Go: internal/rules/unsafe_effect_type_assertion.go

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



#[derive(Clone, Debug)]
pub struct UnsafeEffectTypeAssertionChannel {
    pub name: String,
    pub original: String,
    pub asserted: String,
}

#[derive(Clone, Debug)]
pub struct UnsafeEffectTypeAssertionMatch {
    pub source_file: Node,
    pub location: TextRange,
    pub assertion_node: Node,
    pub expression_node: Node,
    pub location_node: Node,
    pub channels: Vec<UnsafeEffectTypeAssertionChannel>,
}

// Go: rules/unsafe_effect_type_assertion.go AnalyzeUnsafeEffectTypeAssertion

// Go: rules/unsafe_effect_type_assertion.go isAnyType
// PORT: takes the checker to read the type flags.

// Go: rules/unsafe_effect_type_assertion.go unsafeEffectTypeAssertionRelatedInformation
