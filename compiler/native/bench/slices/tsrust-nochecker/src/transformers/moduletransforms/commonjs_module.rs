//! Port of Go `transformers/moduletransforms/commonjsmodule.go` lines 1 to 1037:
//! `NewCommonJSModuleTransformer` through `visitTopLevelNestedBlock`. The
//! rest of the file is in `commonjs_module_p2.rs`.
//!
//! PORT: Go builds five visitors in `NewCommonJSModuleTransformer`
//! (`Visitor()`, `topLevelVisitor`, `topLevelNestedVisitor`,
//! `discardedValueVisitor`, `assignmentPatternVisitor`). Each is an
//! `EmitContext.NewNodeVisitor` over one visit method. Here `with_visitor`
//! builds the visitor named by `VisitorKind` on each use, with the
//! transformer as the visitor context, as the declaration transform does.

use super::external_module_info::{
    ExternalModuleInfo, 
    
    
};
use super::utilities::{
    
    
    
};
use crate::frontend::tspath;
use crate::prelude::*;
use crate::transformers::transformer::{
    TransformReferenceResolver, 
};
use crate::transformers::utilities::{
    
    
};

/// Names the Go visitor fields of `CommonJSModuleTransformer`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum VisitorKind {
    /// Go `tx.Visitor()` (`tx.visit`).
    Root,
    /// Go `tx.topLevelVisitor` (`tx.visitTopLevel`): visits statements at top level of a module
    TopLevel,
    /// Go `tx.topLevelNestedVisitor` (`tx.visitTopLevelNested`): visits nested statements at top level of a module
    TopLevelNested,
    /// Go `tx.discardedValueVisitor` (`tx.visitDiscardedValue`): visits expressions whose values would be discarded at runtime
    DiscardedValue,
    /// Go `tx.assignmentPatternVisitor` (`tx.visitAssignmentPattern`): visits assignment patterns in a destructuring assignment
    AssignmentPattern,
}

// Go: transformers/moduletransforms/commonjsmodule.go:15 CommonJSModuleTransformer

// Go: transformers/moduletransforms/commonjsmodule.go:32 NewCommonJSModuleTransformer


