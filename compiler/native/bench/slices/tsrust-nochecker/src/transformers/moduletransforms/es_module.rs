//! Port of Go `transformers/moduletransforms/esmodule.go`.
//!
//! PORT: Go `tx.Visitor()` is the root visitor built by `NewTransformer` with
//! `EmitContext.NewNodeVisitor(tx.visit)`. Here `with_visitor` builds that
//! visitor over the transformer, as the declaration transform does. Its
//! callback `visit_in` gets the running visitor, so the default arm and plain
//! call expressions visit their children on it instead of building a new
//! visitor for each node. The visitor has no state of its own, so this is the
//! same as Go's one shared `tx.Visitor()`.

use super::utilities::{
    
    
};
use crate::prelude::*;
use crate::transformers::transformer::{
    TransformReferenceResolver, 
};

// Go: transformers/moduletransforms/esmodule.go:13 ESModuleTransformer

// Go: transformers/moduletransforms/esmodule.go:23 importRequireStatements
struct ImportRequireStatements {
    statements: Vec<Node>,
    require_helper_name: Node,
}

// Go: transformers/moduletransforms/esmodule.go:28 NewESModuleTransformer



// Go: transformers/moduletransforms/esmodule.go:35 ESModuleTransformer.visit

// Go: transformers/moduletransforms/esmodule.go:248 ESModuleTransformer.visitCallExpression
