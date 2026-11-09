//! Port of typescript-go `internal/checker/checker.go` lines 13025-13928.

use crate::prelude::*;

// PORT: Go `checkObjectLiteral` builds its result with a closure
// (`createObjectLiteralType`) that reads these locals by reference. The
// locals live in this struct so the closure can become a method.

