//! Port of Go `transformers/destructuring.go`.
//!
//! PORT: Go passes `tx *Transformer` for its emit context, factory and root
//! visitor. Here the caller passes the emit context and its root visitor
//! (`&mut NodeVisitor<'a, C>`, usually from `TransformerVisit::with_visitor`).
//! Go `f.tx.Visitor().VisitNode(n)` is `v.visit_node(n)`. A Go
//! `CreateAssignmentCallback` closes over its transformer; here it gets the
//! visitor, so it reaches the transformer through `v.ctx`.

use crate::prelude::*;

use super::utilities::{};

// Go: transformers/destructuring.go:12 FlattenLevel
// FlattenLevel controls how deeply binding/assignment patterns are decomposed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FlattenLevel {
    /// Fully decompose all patterns into individual assignments/bindings
    All,
    /// Only decompose patterns containing object rest elements
    ObjectRest,
}

// Go: transformers/destructuring.go:22 CreateAssignmentCallback

// Go: transformers/destructuring.go:27 FlattenDestructuringAssignment

// Go: transformers/destructuring.go:46 pendingDecl
// pendingDecl tracks a pending variable declaration during binding flattening.
struct PendingDecl {
    pending_expressions: Vec<Node>,
    name: Node,
    value: Node,
    location: TextRange,
    original: Node,
}

// Go: transformers/destructuring.go:57 FlattenDestructuringBinding

/// Go sets four mode callbacks on the flattener. They always come in one of
/// two sets, so the set is this enum.
// PORT: Go `emitBindingOrAssignment`, `createArrayBindingOrAssignmentPattern`,
// `createObjectBindingOrAssignmentPattern` and
// `createArrayBindingOrAssignmentElement` func fields dispatch on `mode`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Assignment,
    Binding,
}

// Go: transformers/destructuring.go:77 flattener
// flattener encapsulates the state and logic for flattening destructuring patterns.
// It is equivalent to TypeScript's FlattenContext in destructuring.ts.

// Go: transformers/destructuring.go:96 newFlattener


// --- Exported helper functions ---

// Go: transformers/destructuring.go:420 BindingOrAssignmentElementAssignsToName
// BindingOrAssignmentElementAssignsToName checks if any target in a binding/assignment pattern assigns to the given name.

// Go: transformers/destructuring.go:433 bindingOrAssignmentPatternAssignsToName

// Go: transformers/destructuring.go:444 BindingOrAssignmentElementContainsNonLiteralComputedName
// BindingOrAssignmentElementContainsNonLiteralComputedName checks if any element has a non-literal computed property name.

// Go: transformers/destructuring.go:453 bindingOrAssignmentPatternContainsNonLiteralComputedName

// Go: transformers/destructuring.go:459 GetInitializerOfBindingOrAssignmentElement
// GetInitializerOfBindingOrAssignmentElement returns the initializer/default value of a binding or assignment element.

// Go: transformers/destructuring.go:485 isObjectBindingOrAssignmentPattern

// Go: transformers/destructuring.go:489 isArrayBindingOrAssignmentPattern

// Go: transformers/destructuring.go:493 isSimpleBindingOrAssignmentElement
