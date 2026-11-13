//! Port of Effect-TS/tsgo `internal/rules/effect_in_void_success.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

// Go: rules.EffectInVoidSuccess


// Go: rules.findFirstStrictEffect
