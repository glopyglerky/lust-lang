//! Port of Go `checker/nodebuilder.go`: the `NodeBuilder` entry points that
//! push a context, call into `NodeBuilderImpl` and pop the context.
//!
//! PORT: Go `NodeBuilder` methods reach the checker through `b.impl.ch`. Here
//! the entry points are `impl Checker` methods named `node_builder_<name>`
//! that take `nb: &Rc<RefCell<NodeBuilder>>`. Methods that do not need the
//! checker stay on `NodeBuilder`.

use crate::prelude::*;
use crate::printer::{};

// Go: checker/nodebuilder.go:10 NodeBuilder
// PORT: Go `VerbosityContext` (line 21) is defined in printer_impl.rs, with
// shared cells for the two output fields.


// Go: checker/nodebuilder.go:188 simplifyClassDeclaration

// Go: checker/nodebuilder.go:212 simplifyModifiers



// Go: checker/nodebuilder.go:279 NewNodeBuilder

// Go: checker/nodebuilder.go:283 NewNodeBuilderEx
