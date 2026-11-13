//! Port of Go `checker/symboltracker.go`.

use crate::prelude::*;
use std::cell::Cell;
use std::rc::Weak;

// Go: checker/symboltracker.go:8 SymbolTrackerImpl
// PORT: Go `context` is a plain pointer. The context also holds this tracker,
// so here it is a `Weak` to break the `Rc` cycle. The context outlives every
// call on its tracker, so the upgrade does not fail.

// Go: checker/symboltracker.go:14 NewSymbolTrackerImpl
// PORT: returns an `Rc`, the shape that `NodeBuilderContext.tracker` holds.


