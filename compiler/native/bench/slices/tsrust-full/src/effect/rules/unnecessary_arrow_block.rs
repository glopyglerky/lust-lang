//! Port of Effect-TS/tsgo `internal/rules/unnecessary_arrow_block.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



// Go: rules/unnecessary_arrow_block.go UnnecessaryArrowBlockMatch
#[derive(Clone, Debug)]
pub struct UnnecessaryArrowBlockMatch {
    pub source_file: Node,
    pub location: TextRange,
    pub arrow_function: Node,
    pub body_block: Node,
    pub returned_expression: Node,
}

// Go: rules/unnecessary_arrow_block.go AnalyzeUnnecessaryArrowBlock

// Go: rules/unnecessary_arrow_block.go analyzeUnnecessaryArrowBlockNode
