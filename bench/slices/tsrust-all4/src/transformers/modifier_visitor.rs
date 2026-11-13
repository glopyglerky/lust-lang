//! Port of Go `transformers/modifiervisitor.go`.

use crate::prelude::*;


// Go: transformers/modifiervisitor.go:13 modifierVisitor.visit
// PORT: Go `modifierVisitor` only holds `AllowedModifiers`; it is the
// visitor context here.

// Go: transformers/modifiervisitor.go:21 ExtractModifiers
