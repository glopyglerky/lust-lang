//! Port of checker/nodebuilderimpl.go lines 3056 to 3552: visitAndTransformType,
//! typeToTypeNode and the small node helpers that follow it.
//!
//! Relater pattern (printer plan contract item 8): NodeBuilderImpl methods are
//! `impl Checker` methods that take `b: &Rc<RefCell<NodeBuilderImpl>>`. No
//! `RefCell` borrow is held across a Checker call.

use crate::prelude::*;
use crate::printer::{EmitFlags};

// PORT: private accessors for NodeBuilderImpl fields. They keep the field
// shapes (owned by U11) in one place. Clone the handle out, then drop the
// borrow before any Checker call.






