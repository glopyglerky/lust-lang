//! Port of typescript-go `internal/checker/checker.go` lines 13025-13928.

use crate::gostd::Context;
use crate::prelude::*;

// PORT: Go `checkObjectLiteral` builds its result with a closure
// (`createObjectLiteralType`) that reads these locals by reference. The
// locals live in this struct so the closure can become a method.
struct ObjectLiteralBuildState {
    node: Node,
    contextual_type: TypeId,
    in_destructuring_pattern: bool,
    properties_table: SymbolTable,
    properties_array: Vec<SymbolId>,
    object_flags: ObjectFlags,
    pattern_with_computed_properties: bool,
    has_computed_string_property: bool,
    has_computed_number_property: bool,
    has_computed_symbol_property: bool,
    offset: usize,
}

