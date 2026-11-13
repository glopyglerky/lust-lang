//! Port of Effect-TS/tsgo `internal/rules/schema_literal_non_finite.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

// Go: rules/schema_literal_non_finite.go SchemaLiteralNonFinite

