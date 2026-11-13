//! Port of `checker/checker.go` lines 29672-30606: contextual types for
//! object literal elements, array elements, conditionals and templates,
//! effective call arguments, decorator call signatures, contextual property
//! types, apparent contextual types and object literal discrimination.

use crate::jsnum::Number;
use crate::prelude::*;







// Go: checker/checker.go:30596 isSpreadArgument


// Go: checker/checker.go:31197 ObjectLiteralDiscriminator
// PORT: Go keeps a `c *Checker` field. The Rust `Discriminator` trait
// methods receive the checker instead, so the struct only holds the items.
#[derive(Clone, Debug, Default)]
pub struct ObjectLiteralDiscriminator {
    pub props: Vec<Node>,
    pub members: Vec<SymbolId>,
}


