//! Port of Go `ls/lsutil/children.go`.

use crate::astnav;
use crate::frontend::scanner::scanner_ls;
use crate::ls::lsutil::prelude::*;
use std::cell::Cell;

// Go: ls/lsutil/children.go:13 GetLastChild

// Go: ls/lsutil/children.go:37 GetLastToken

// Go: ls/lsutil/children.go:62 GetLastVisitedChild

// Go: ls/lsutil/children.go:87 GetFirstToken

// Go: ls/lsutil/children.go:128 AssertHasRealPosition
