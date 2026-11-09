//! Port of `checker/checker.go` lines 29672-30606: contextual types for
//! object literal elements, array elements, conditionals and templates,
//! effective call arguments, decorator call signatures, contextual property
//! types, apparent contextual types and object literal discrimination.

use crate::jsnum::Number;
use crate::prelude::*;

/// The result of `get_effective_call_arguments`: Go `[]*ast.Node`.
// PERF: callcopy1. Go returns the call's own argument list without a copy.
// `Slice` is that list; it is `Copy`, so a caller that needs only a length
// or a position reads it in place. The other cases build a new list.
#[derive(Debug)]
pub enum EffectiveArgs {
    /// The call's arguments, with no spread (`node.arguments()`).
    Slice(NodeSlice),
    /// Synthetic arguments: spreads of tuple types, tagged templates, JSX,
    /// decorators and `instanceof`.
    Owned(Vec<Node>),
}

impl EffectiveArgs {
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Self::Slice(args) => args.len(),
            Self::Owned(args) => args.len(),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }


}

/// Iterator over `EffectiveArgs`. Yields `Node` by value.
pub enum EffectiveArgsIter<'a> {
    Slice(NodeSliceIter),
    Owned(std::slice::Iter<'a, Node>),
}




// Go: checker/checker.go:30596 isSpreadArgument


// Go: checker/checker.go:31197 ObjectLiteralDiscriminator
// PORT: Go keeps a `c *Checker` field. The Rust `Discriminator` trait
// methods receive the checker instead, so the struct only holds the items.
#[derive(Clone, Debug, Default)]
pub struct ObjectLiteralDiscriminator {
    pub props: Vec<Node>,
    pub members: Vec<SymbolId>,
}


