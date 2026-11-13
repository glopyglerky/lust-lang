//! Port of Go `checker/nodebuilderimpl.go` lines 1 to 1018: the node builder
//! types, the truncation and expansion helpers, and symbol to name/node
//! conversion.
//!
//! Relater pattern (printer plan contract item 8): NodeBuilderImpl methods are
//! `impl Checker` methods that take `b: &Rc<RefCell<NodeBuilderImpl>>`. No
//! `RefCell` borrow is held across a Checker call or a tracker call.

use crate::prelude::*;
use crate::printer::{EmitFlags, SymbolAccessibility};
use crate::pseudochecker::{PseudoChecker, new_pseudo_checker};
use std::cell::Cell;
use std::rc::Weak;

// Go: checker/nodebuilderimpl.go:24 CompositeSymbolIdentity
// PORT: Go `ast.SymbolId` and `ast.NodeId` are the u64 values that
// `get_symbol_id` and `get_node_id` return.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CompositeSymbolIdentity {
    pub is_constructor_node: bool,
    pub symbol_id: u64,
    pub node_id: u64,
}

// Go: checker/nodebuilderimpl.go:30 TrackedSymbolArgs
// PORT: Go stores `*TrackedSymbolArgs`. The struct is small and never
// mutated after creation, so it is stored by value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrackedSymbolArgs {
    pub symbol: SymbolId,
    pub enclosing_declaration: Node,
    pub meaning: SymbolFlags,
}

// Go: checker/nodebuilderimpl.go:36 SerializedTypeEntry
#[derive(Clone, Debug)]
pub struct SerializedTypeEntry {
    pub node: Node,
    pub truncating: bool,
    pub added_length: i32,
    pub tracked_symbols: Vec<TrackedSymbolArgs>,
}

// Go: checker/nodebuilderimpl.go:43 CompositeTypeCacheIdentity
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CompositeTypeCacheIdentity {
    pub type_id: TypeId,
    pub flags: NodeBuilderFlags,
    pub internal_flags: InternalNodeBuilderFlags,
}

// Go: checker/nodebuilderimpl.go:49 NodeBuilderLinks
// PORT: the Go map is created on first use. Here it always exists and starts
// empty, which reads the same.
#[derive(Clone, Debug, Default)]
pub struct NodeBuilderLinks {
    pub serialized_types: FxHashMap<CompositeTypeCacheIdentity, SerializedTypeEntry>, // Collection of types serialized at this location
    pub fake_scope_for_signature_declaration: Option<String>, // If present, this is a fake scope injected into an enclosing declaration chain.
}

// Go: checker/nodebuilderimpl.go:54 NodeBuilderSymbolLinks
// PORT: Go `module.ModeAwareCache[moduleSpecifierResult]` is a map keyed by
// `(path, resolution mode)`. `None` is the nil cache.
#[derive(Clone, Debug, Default)]
pub struct NodeBuilderSymbolLinks {
    pub specifier_cache: Option<FxHashMap<(String, ResolutionMode), ModuleSpecifierResult>>,
}

// Go: checker/nodebuilderimpl.go:58 moduleSpecifierResult
// PORT: a nil `importAttributesType` is `TypeId::NIL`.
#[derive(Clone, Debug, Default)]
pub struct ModuleSpecifierResult {
    pub specifier: String,
    pub import_attributes_type: TypeId,
}

// Go: checker/nodebuilderimpl.go:63 NodeBuilderContext
// PORT: symbol keyed maps use `SymbolId` for Go `ast.SymbolId`. A symbol id
// is the same for the same symbol, so the maps behave the same. Go
// `*ast.SourceFile` is the `SourceFile` node. Go `Host` is the program.




// Go: checker/nodebuilderimpl.go:97 NodeBuilderImpl
// PORT: Go `ch *Checker` is dropped. Every method is a Checker method and
// gets the checker as `self`. Go `f` is the method `f()`, which returns the
// emit context factory. Go `cloneBindingNameVisitor` is dropped: nothing in
// the ported code reads it, and the visitor would need the checker.


// Go: checker/nodebuilderimpl.go:112 defaultMaximumTruncationLength
pub const DEFAULT_MAXIMUM_TRUNCATION_LENGTH: i32 = 160;
// Go: checker/nodebuilderimpl.go:112 noTruncationMaximumTruncationLength
pub const NO_TRUNCATION_MAXIMUM_TRUNCATION_LENGTH: i32 = 1_000_000;

// Node builder utility functions

// Go: checker/nodebuilderimpl.go:125 newNodeBuilderImpl

// PORT: private accessors. Clone the handle out, then drop the borrow before
// any Checker or tracker call.




// Go: checker/nodebuilderimpl.go:325 getAccessStack

// Go: checker/nodebuilderimpl.go:463 isIdentifierTypeReference

// Go: checker/nodebuilderimpl.go:467 arrayIsHomogeneous
fn array_is_homogeneous<T>(array: &[T], comparer: impl Fn(&T, &T) -> bool) -> bool {
    if array.len() < 2 {
        return true;
    }
    let first = &array[0];
    for target in &array[1..] {
        if !comparer(first, target) {
            return false;
        }
    }
    true
}

// Go: checker/nodebuilderimpl.go:745 getTopmostIndexedAccessType

// Go: checker/nodebuilderimpl.go:912 canUsePropertyAccess

// Go: checker/nodebuilderimpl.go:924 startsWithSingleOrDoubleQuote
fn starts_with_single_or_double_quote(str: &str) -> bool {
    str.starts_with('\'') || str.starts_with('"')
}

// Go: checker/nodebuilderimpl.go:928 startsWithSquareBracket
fn starts_with_square_bracket(str: &str) -> bool {
    str.starts_with('[')
}

// Go: checker/nodebuilderimpl.go:932 isDefaultBindingContext

