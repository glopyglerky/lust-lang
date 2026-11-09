//! Port of Effect-TS/tsgo `internal/rules/catch_all_tag_dispatch_to_catch_tag.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

use super::catch_tag_to_catch_reason::{
    
};



#[derive(Clone, Debug, Default)]
pub struct CatchAllTagDispatchBranch {
    pub tag: String,
    pub result: Node,
    pub uses_parameter: bool,
}






