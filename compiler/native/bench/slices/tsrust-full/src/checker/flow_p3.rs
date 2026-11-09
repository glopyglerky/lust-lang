//! Port of `checker/flow.go` lines 1855-2734: reference roots, switch
//! exhaustiveness, effects signatures, dotted names, initial and assigned
//! types, reachability, post-super analysis and assignment marking.

use crate::prelude::*;

