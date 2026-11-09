//! Port of Effect-TS/tsgo `internal/rules/flat_map_conditional_to_filter_or_fail.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;





// Go: rules/flat_map_conditional_to_filter_or_fail.go isIdentitySucceed

// Go: rules/flat_map_conditional_to_filter_or_fail.go effectFailArgument
