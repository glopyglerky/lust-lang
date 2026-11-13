//! Port of Go `checker/checker.go` lines 24046-24956 (checker-27):
//! tuple, union, intersection, template, mapped, conditional, infer and
//! import type nodes, conditional type resolution, tuple targets, generic
//! object flags, and the basic type constructors (`newType` ...
//! `newLiteralType`).

use crate::jsnum::Number;
use crate::prelude::*;
use crate::smallvec::SmallVec;


// Go: checker/checker.go:25464 isUnaryTupleTypeNode


