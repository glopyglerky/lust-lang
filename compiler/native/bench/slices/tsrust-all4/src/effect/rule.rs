//! Port of Effect-TS/tsgo `internal/rule`: the rule type and the context a
//! rule runs with.

use crate::effect::directives::to_category;
use crate::effect::etscore::{Severity};
use crate::gostd::Context;
use crate::prelude::*;


pub const UNUSED_DIRECTIVE_NAME: &str = "unusedDirective";
pub const UNKNOWN_RULE_NAME_NAME: &str = "unknownRuleName";

// Go: rule.ByName

// Go: rule.CodeToRuleName


