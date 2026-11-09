//! Port of Effect-TS/tsgo `internal/rules/redundant_map_error.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

// Go: rules/redundant_map_error.go RedundantMapError


// Go: rules/redundant_map_error.go RedundantGeneratorMapErrorMatch
#[derive(Clone)]
pub struct RedundantGeneratorMapErrorMatch {
    pub source_file: Node,
    pub location: TextRange,
    pub generator_call_node: Node,
    pub generator_function: Node,
    pub map_error_nodes: Vec<Node>,
    pub mapper_node: Node,
    pub yield_expressions: Vec<Node>,
}

// Go: rules/redundant_map_error.go redundantMapErrorCandidate
#[derive(Clone, Default)]
pub struct RedundantMapErrorCandidate {
    pub yield_expression: Node,
    pub map_error_node: Node,
    pub mapper_node: Node,
    pub mapper_text: String,
}

// Go: rules/redundant_map_error.go AnalyzeRedundantGeneratorMapError

// Go: rules/redundant_map_error.go analyzeRedundantMapErrorFlowCandidate

// Go: rules/redundant_map_error.go analyzeRedundantGeneratorMapErrorCandidate
// PORT: Go returns a zero match with `false`; the port returns `None` there.

// Go: rules/redundant_map_error.go redundantMapErrorRelatedInformation

// Go: rules/redundant_map_error.go enclosingYieldExpression

// Go: rules/redundant_map_error.go enclosingGeneratorCall

// Go: rules/redundant_map_error.go mapperCanBeHoisted

// Go: rules/redundant_map_error.go allSymbolDeclarationsInsideNode

// Go: rules/redundant_map_error.go allSymbolDeclarationsOutsideNode

// Go: rules/redundant_map_error.go hoistSensitiveReferenceSymbol

// Go: rules/redundant_map_error.go generatorFunctionKeywordRange

// Go: rules/redundant_map_error.go isHoistSensitiveValueReference

// Go: rules/redundant_map_error.go unwrapTransparentExpression

// Go: rules/redundant_map_error.go nodeSourceText
