//! Port of Effect-TS/tsgo `internal/effectconfigcheck`: reports rule names
//! in `diagnosticSeverity` that no rule has (`unknownRuleName`).

use crate::effect::diag;
use crate::effect::directives::to_category;
use crate::effect::etscore::{EFFECT_PLUGIN_NAME, Severity, };
use crate::effect::rule::{UNKNOWN_RULE_NAME_NAME, UNUSED_DIRECTIVE_NAME, };
use crate::effect::rules;
use crate::prelude::*;

// Go: effectconfigcheck.validate




