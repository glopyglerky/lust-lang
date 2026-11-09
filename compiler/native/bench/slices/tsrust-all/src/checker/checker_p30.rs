//! Port of typescript-go `internal/checker/checker.go` lines 26861-27804.

use crate::jsnum::Number;
use crate::prelude::*;
use crate::smallvec::SmallVec;



// Go: checker/checker.go:27722 getIndexNodeForAccessExpression


// Go: checker/checker.go:28164 compareTypesEqual
// PORT: a free function because it only compares handles. Pass it as
// `&mut |_: &mut Checker, s, t| compare_types_equal(s, t)`.
pub fn compare_types_equal(s: TypeId, t: TypeId) -> Ternary {
    if s == t {
        return Ternary::TRUE;
    }
    Ternary::FALSE
}


// Go: checker/checker.go:28252 hasRestParameter

// Go: checker/checker.go:28257 isRestParameter

