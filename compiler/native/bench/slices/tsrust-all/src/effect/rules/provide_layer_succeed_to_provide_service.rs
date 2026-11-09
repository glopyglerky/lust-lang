//! Port of Effect-TS/tsgo `internal/rules/provide_layer_succeed_to_provide_service.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;





// PORT: Go returns (service, implementation, replacement, ok); the port
// returns `None` for `ok == false`.
