//! Port of Go `checker/checker.go` lines 9355-10268.

use crate::prelude::*;
use crate::smallvec::SmallVec;

// PORT: Go `core.FirstOrNil` / `core.ElementOrNil` over a `NodeSlice`.


// Go: checker/checker.go:10035 getErrorNodeForCallNode

