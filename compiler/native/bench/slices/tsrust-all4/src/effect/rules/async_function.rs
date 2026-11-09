//! Port of Effect-TS/tsgo `internal/rules/async_function.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

// Go: rules/async_function.go AsyncFunction

