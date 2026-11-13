//! Port of Effect-TS/tsgo `internal/rules/missing_effect_service_dependency.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



// Go: the `excludeNever` closure of checkServiceDependencies.

