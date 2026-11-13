// Go: internal/rules/flat_map_to_map.go

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



/// FlatMapToMapMatch holds the AST nodes needed by the diagnostic and quick fix.
#[derive(Clone, Debug)]
pub struct FlatMapToMapMatch {
    pub source_file: Node,
    pub location: TextRange,
    pub callee: Node,
    pub callee_name_node: Node,
    pub succeed_call_expression: Node,
    pub succeed_argument: Node,
}

// Go: rules/flat_map_to_map.go AnalyzeFlatMapToMap
