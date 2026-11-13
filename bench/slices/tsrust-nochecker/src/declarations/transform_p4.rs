//! Port of `transformers/declarations/transform.go` lines 2153 to 2939
//! (`walkBindingPattern` to the end of the file).
//!
//! PORT: this file is a child module of `declarations::transform` (like
//! `transform_p3.rs`), so it can read the private `DeclarationTransformer`
//! fields and helpers.
//!
//! PORT: Go keeps `*ast.NodeVisitor` fields made by
//! `EmitContext.NewNodeVisitor` (bindingNameVisitor, expressionVisitor,
//! cjsExportAssignmentVisitor). Each use here builds one on demand with
//! `EmitContext::new_node_visitor` and the same callback (`with_tx_visitor`),
//! so the emit context hooks are attached as in Go. The two whole-tree walks
//! (`visit_cjs_export_assignments`, `visit_nested_expression`) build one
//! visitor per walk and recurse through it. Go
//! `tx.Visitor().Visit(n)` calls the callback directly, so it is `self.visit(n)`.
//!
//! PORT: Go `setupDiagnosticContext` returns a cleanup closure. Here it
//! returns `CleanupDiagnosticContext`, and `cleanup.run(self)` restores it. Go
//! runs the cleanup in a `defer`, so every return path below runs it.

use crate::ast::visitor::{};
use crate::declarations::diagnostics::{
    
};
use crate::declarations::transform::{
    
    
};
use crate::declarations::util::{
    
    
    
};
use crate::prelude::*;




// Go: transformers/declarations/transform.go:2680 DeclarationTransformer.visitCJSExportAssignments

// Go: transformers/declarations/transform.go:2700 DeclarationTransformer.visitNestedExpression



// Go: transformers/declarations/transform.go:2661 isNotDeclareModifier

// Go: transformers/declarations/transform.go:2973 extractExpandoHostParams
// Returns (typeParameters, parameters, asteriskToken).
