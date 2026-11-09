//! Port of Effect-TS/tsgo `internal/rules/schema_sync.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

// SchemaSync discourages synchronous Schema decoding and encoding in any context.

