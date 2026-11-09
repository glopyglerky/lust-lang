//! Port of checker/nodebuilder_hover.go: declaration nodes for expandable hover.
//!
//! Every NodeBuilderImpl method uses the Relater pattern: an `impl Checker`
//! method that takes `b: &Rc<RefCell<NodeBuilderImpl>>`. No `RefCell` borrow is
//! held across a Checker call.

use crate::prelude::*;

// Go: checker/nodebuilder_hover.go:16 isExpanding

// PORT: Go repeats `b.f.NewModifierList(ast.CreateModifiersFromModifierFlags(flags, b.f.NewModifier))`
// inline. This helper is that expression.

// PORT: Go reads `b.ctx` directly. The context is its own RefCell here, so
// clone the handle out and drop the builder borrow before any Checker call.


// Go: checker/nodebuilder_hover.go:217 typeElementsToClassElements

// Go: checker/nodebuilder_hover.go:603 isHashPrivate
