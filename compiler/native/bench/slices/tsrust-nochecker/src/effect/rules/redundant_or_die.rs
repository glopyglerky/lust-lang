//! Port of Effect-TS/tsgo `internal/rules/redundant_or_die.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

// Go: rules/redundant_or_die.go RedundantOrDie


// Go: rules/redundant_or_die.go RedundantGeneratorOrDieMatch
#[derive(Clone, Debug)]
pub struct RedundantGeneratorOrDieMatch {
    pub source_file: Node,
    pub location: TextRange,
    pub generator_call_node: Node,
    pub generator_function: Node,
    pub or_die_nodes: Vec<Node>,
    pub yield_expressions: Vec<Node>,
}

// Go: rules/redundant_or_die.go redundantOrDieCandidate
#[derive(Clone, Copy, Debug, Default)]
pub struct RedundantOrDieCandidate {
    pub yield_expression: Node,
    pub or_die_node: Node,
}

// PORT: Go returns nil for a nil `tp` or checker; the port has neither.
// Go ranges over `candidatesByGenerator` in random map order; the port keeps
// the first-seen order of the generator functions (`IndexMap`).
// Go: rules/redundant_or_die.go AnalyzeRedundantGeneratorOrDie

// Go: rules/redundant_or_die.go analyzeRedundantOrDieFlowCandidate

// Go: rules/redundant_or_die.go analyzeRedundantGeneratorOrDieCandidate

// Go: rules/redundant_or_die.go redundantOrDieRelatedInformation
