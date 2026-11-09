//! Port of Effect-TS/tsgo `internal/rules/multiple_catch_tag.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



#[derive(Clone, Debug)]
pub struct MultipleCatchTagMatch {
    pub source_file: Node,
    pub location: TextRange,
    pub node: Node,
    pub catch_nodes: Vec<Node>,
}

// PORT: Go keeps a pointer into the flow's transformations; the port keeps
// a copy of the transformation.







