//! Port of Effect-TS/tsgo `internal/rules/schema_opaque_instance_member.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

// Go: rules/schema_opaque_instance_member.go SchemaOpaqueInstanceMember


// Go: rules/schema_opaque_instance_member.go isSchemaOpaqueInstanceMember
