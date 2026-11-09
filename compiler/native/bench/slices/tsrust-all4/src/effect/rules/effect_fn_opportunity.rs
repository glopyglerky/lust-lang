//! Port of Effect-TS/tsgo `internal/rules/effect_fn_opportunity.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



/// EffectFnOpportunityMatch holds the parsed result needed by both the diagnostic rule
/// and the quick-fix for the effectFnOpportunity pattern.
// Go: rules/effect_fn_opportunity.go EffectFnOpportunityMatch
#[derive(Clone)]
pub struct EffectFnOpportunityMatch {
    pub source_file: Node,
    pub location: TextRange,
    pub result: Rc<EffectFnOpportunityResult>,
}





