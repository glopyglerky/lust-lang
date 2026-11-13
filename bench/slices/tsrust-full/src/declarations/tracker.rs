//! Port of `transformers/declarations/tracker.go`.

use std::cell::Cell;

use super::diagnostics::{
    
    
};
use crate::prelude::*;
use crate::printer::{SymbolAccessibility, SymbolAccessibilityResult};

// Go: transformers/declarations/tracker.go:12 SymbolTrackerImpl
// PORT: the checker `SymbolTracker` trait takes `&self`, so the fields that Go
// changes in place are `Cell`/`RefCell`. The name shadows the checker's own
// `SymbolTrackerImpl` from the prelude glob; this module means this one.



// Go: transformers/declarations/tracker.go:239 SymbolTrackerSharedState


// Go: transformers/declarations/tracker.go:255 NewSymbolTracker
