//! Port of `transformers/declarations/transform.go` lines 753 to 1486:
//! `transformImportTypeNode` through `transformCommonJSExportWorker`.
//!
//! PORT: Go `tx.Visitor()` is the root visitor that `NewTransformer` builds
//! with `EmitContext.NewNodeVisitor(tx.visit)`. Its `Visit` field is
//! `tx.visit`, so Go `tx.Visitor().Visit(n)` is `self.visit(n)` here. The
//! other visitor methods (`VisitNode`, `VisitNodes`, `VisitEachChild`) go
//! through `with_visitor`, which builds an `ast` visitor over the transformer.
//! `with_visitor` uses `EmitContext::new_node_visitor`, so the emit context
//! hooks (`VisitParameters`, `VisitFunctionBody`, ...) are attached as in Go.
//!
//! Methods ported in other files of this module (`rewriteModuleSpecifier`,
//! `ensureType`, `ensureModifiers`, `preserveJsDoc`, ...) are called with their
//! Go snake names.

use super::diagnostics::{};
use super::util::{
    
};
use crate::ast::visitor::{};
use crate::prelude::*;
use crate::printer::{
    AutoGenerateOptions, GeneratedIdentifierFlags, SymbolAccessibilityResult,
};

/// Go `printer.AutoGenerateOptions{Flags: printer.GeneratedIdentifierFlagsOptimistic}`.
fn optimistic() -> AutoGenerateOptions {
    AutoGenerateOptions {
        flags: GeneratedIdentifierFlags::OPTIMISTIC,
        ..Default::default()
    }
}





// Go: transformers/declarations/transform.go:859 hasAnyBindingInitializers
