// Go: internal/typeparser/schema_type.go

use crate::effect::typeparser::*;
use crate::prelude::*;
use std::sync::LazyLock;




/// SchemaTypeId is the property key for Schema's variance struct.
pub const SCHEMA_TYPE_ID: &str = "~effect/Schema/Schema";

/// SchemaTypes holds the A (Type) and E (Encoded) types extracted from a Schema type.
#[derive(Clone, Debug)]
pub struct SchemaTypes {
    pub a: TypeId,
    pub e: TypeId,
}


// Go: typeparser/schema_type.go isSchemaTypeSourceFile


// Go: typeparser/schema_type.go isParseResultSourceFile


// Go: typeparser/schema_type.go isSchemaParserSourceFile

