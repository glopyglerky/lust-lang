//! Go `checker/nodebuilderimpl.go` lines 2156 to 3055: type serialization
//! for declarations, object type element lists, anonymous types,
//! conditional types and type references.
//!
//! Methods use the Relater pattern: `impl Checker` methods take `&mut self`
//! and `b: &Rc<RefCell<NodeBuilderImpl>>`. No `RefCell` borrow of `b` is held
//! across a Checker call.

use crate::prelude::*;
use crate::printer::{EmitFlags, SymbolAccessibility};
use crate::pseudochecker::{self, PseudoChecker, PseudoTypeKind};

// PORT: the helpers below are the only places that read the layout of
// `NodeBuilderImpl` (fields `e`, `pc`, `ctx`) and of `SymbolTracker`. If
// that layout changes, only these helpers change. They are free functions so
// they do not clash with methods of other node builder files.





// PORT: `SymbolTracker` methods take the checker (see nodebuilder_types.rs),
// so these wrappers take it too. The tracker `Rc` is cloned out first, so no
// borrow of `b` is held during the call.






// Go: checker/nodebuilderimpl.go:2372 MAX_REVERSE_MAPPED_NESTING_INSPECTION_DEPTH
pub const MAX_REVERSE_MAPPED_NESTING_INSPECTION_DEPTH: usize = 3;

// Go: checker/nodebuilderimpl.go:2448 propertyNameNodeKind
// PORT: the Go type is `PropertyNameNodeKind` in `crate::flags`.

// Go: checker/nodebuilderimpl.go:2456 classifyPropertyName

