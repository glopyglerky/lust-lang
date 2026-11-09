//! Port of Effect-TS/tsgo `internal/rules/strict_boolean_expressions.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;
use std::collections::VecDeque;


