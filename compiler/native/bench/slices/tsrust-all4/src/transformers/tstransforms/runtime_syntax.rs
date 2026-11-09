//! Port of `transformers/tstransforms/runtimesyntax.go`.
//!
//! Transforms TypeScript-specific runtime syntax (enums, namespaces,
//! parameter properties, `import x = N.y`) into JavaScript-compatible syntax.

// !!! SourceMaps and Comments need to be validated

use crate::prelude::*;
use crate::printer::factory::{AssignedNameOptions, NameOptions};
use crate::transformers::destructuring::{FlattenLevel, };
use crate::transformers::transformer::{
    TransformReferenceResolver, 
};
use crate::transformers::utilities::{
    
    
};

/// Go `map[string]*ast.Node` held by pointer: saved and restored by
/// reference, and `nil` until the first record.
type FirstDeclarationsOfName = Option<Rc<RefCell<FxHashMap<String, Node>>>>;

// Go: transformers/tstransforms/runtimesyntax.go:17 RuntimeSyntaxTransformer

// Go: transformers/tstransforms/runtimesyntax.go:31 NewRuntimeSyntaxTransformer
// PORT: Go never returns nil here. The result is an `Option` so the
// constructor has the `TransformerFactory` shape.




/// Go `core.Coalesce(a, b)` for nodes: `a` unless it is nil.
fn coalesce(a: Node, b: Node) -> Node {
    if a.is_some() { a } else { b }
}

// Go: transformers/tstransforms/runtimesyntax.go:990 getInnermostModuleDeclarationFromDottedModule
