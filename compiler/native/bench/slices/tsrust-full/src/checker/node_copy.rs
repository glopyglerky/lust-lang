//! Port of checker/nodecopy.go: reuse of existing type nodes inside the node
//! builder, with a recovery boundary that discards the copy on error.
//!
//! Relater pattern (printer plan contract item 8): NodeBuilderImpl methods are
//! `impl Checker` methods that take `b: &Rc<RefCell<NodeBuilderImpl>>`. No
//! `RefCell` borrow is held across a Checker call.
//!
//! PORT: Go builds the existing-node visitor from closures over `b`, `bound`
//! and `visitor`. Here the captured state is `ExistingNodeTreeState`, the
//! `ctx` of an `ast::NodeVisitor` (see ast/visitor.rs). Go's closures become
//! methods on that visitor type (`ExistingNodeTreeVisitor`), and the visit
//! callback and hooks call them.

use crate::prelude::*;
use crate::printer::{EmitFlags, SymbolAccessibility};

// PORT: private accessors for NodeBuilderImpl fields (owned by U11). Clone
// the handle out, then drop the borrow before any Checker call.




// Go: checker/nodecopy.go:98 recoveryBoundary


// Go: checker/nodecopy.go:116 originalRecoveryScopeState
#[derive(Clone, Copy, Debug)]
pub struct OriginalRecoveryScopeState {
    pub tracked_symbols_top: usize,
    pub unreported_errors_top: usize,
    pub had_error: bool,
}

// Go: checker/nodecopy.go:134 wrappingTracker


// Go: checker/nodecopy.go:188 newWrappingTracker


// Go: checker/emitresolver.go:311 getMeaningOfEntityNameReference
// PORT: no unit owns emitresolver.go yet, so the free function is ported here
// as a private helper. Move it when emitresolver.go lands.

// Go: checker/nodecopy.go:288 getExistingNodeTreeVisitor
// TODO: wrap all these closures into methods on an object so we can guarantee we reuse the same memory on each invocation by reusing/resetting the object
// instead of re-closing-over all of these each time we need a visitor. In theory the compiler could handle this, but in practice closure inlining hasn't been reliable
// PORT: Go's closures are the methods of `ExistingNodeTreeVisitor` below.
// Their captured state is the visitor's `ctx`, and `nonLocalNode` starts true.
// `e` is Go `b.e`; the caller keeps it alive so the visitor can borrow its
// factory (Go `b.f`).





// Go: checker/nodecopy.go:295 visitorFunc (inside attachSymbolToLeftmostIdentifier)

// Go: checker/nodecopy.go:293 attachSymbolToLeftmostIdentifier
// PORT: builds Go `vis` over a reborrow of the checker. See the method below.



