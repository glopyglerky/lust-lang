// Go: internal/rules/effect_do_notation.go

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



#[derive(Clone, Debug)]
pub struct EffectDoNotationMatch {
    pub source_file: Node,
    pub location: TextRange,
    pub node: Node,
}

// Go: rules/effect_do_notation.go AnalyzeEffectDoNotation
