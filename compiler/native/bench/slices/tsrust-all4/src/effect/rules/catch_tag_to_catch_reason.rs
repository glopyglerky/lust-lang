//! Port of Effect-TS/tsgo `internal/rules/catch_tag_to_catch_reason.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

use super::result_dispatch::{};



#[derive(Clone, Debug, Default)]
pub struct CatchTagToCatchReasonBranch {
    pub reason_tag: String,
    pub result: Node,
    pub uses_parameter: bool,
    pub reason_parameter_name: String,
}


#[derive(Clone, Debug, Default)]
pub struct CatchTagToCatchReasonHandler {
    pub parameter_name: String,
    pub branches: Vec<CatchTagToCatchReasonBranch>,
    pub can_fix: bool,
}


// PORT: Go returns `(match, ok)`; the port returns `Some(match)` for ok.











