//! Port of `transformers/declarations/transform.go` lines 1487 to 2152
//! (`wrapInCJSExportNamespace` to `collectThisPropertyAssignments`).
//!
//! PORT: this file is a child module of `declarations::transform`, so it can
//! read the private `DeclarationTransformer` fields and helpers.
//!
//! PORT: Go keeps six `*ast.NodeVisitor` fields made by
//! `EmitContext.NewNodeVisitor`. Each use here builds one on demand with
//! `EmitContext::new_node_visitor` and the same callback (`with_tx_visitor`),
//! so the emit context hooks are attached as in Go. Go
//! `tx.Visitor().Visit(n)` calls the callback directly, so it is
//! `self.visit(n)`.

use crate::ast::visitor::{};
use crate::declarations::diagnostics::{
    
    
};
use crate::declarations::transform::{
    
    ThisPropertyAssignmentKey, 
};
use crate::declarations::util::{
    
    
};
use crate::prelude::*;
use crate::printer::{
    AutoGenerateOptions, EmitFlags, GeneratedIdentifierFlags,
    SymbolAccessibilityResult,
};






// Go: transformers/declarations/transform.go:1562 isCommonJSAliasExport

// Go: transformers/declarations/transform.go:2153 isClassExtendingNull
