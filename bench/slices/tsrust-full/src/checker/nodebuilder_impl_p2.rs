//! Port of `checker/nodebuilderimpl.go` lines 1019 to 2155: type parameter
//! lookup, symbol chains, module specifiers, mapped types, type predicates,
//! parameters, signatures and index infos.
//!
//! Methods follow the NodeBuilder pattern: `impl Checker` methods take
//! `b: &Rc<RefCell<NodeBuilderImpl>>`. No `RefCell` borrow of `b` is held
//! across a Checker call.

use crate::prelude::*;
use crate::printer::{EmitFlags};


// PORT: small readers for `b.ctx` and `b.e`. Each one takes and drops the
// borrow, so callers never hold it across a Checker call.
// Go `b.ctx`. The context is its own `Rc<RefCell<..>>`, so clone the handle
// out and drop the `b` borrow first.







// Go `b.ctx.tracker`. Clone the tracker out so no borrow is held across the
// tracker call.

// PORT: Go `b.e`. Node creation uses `b.e.factory` for Go `b.f`, since the
// printer NodeFactory derefs to the same `ast::NodeFactory`.

// PORT: Go `modulespecifiers.CountPathComponents`. The modulespecifiers
// package is not ported; this is the full Go body.
// Go: modulespecifiers/compare.go:7 CountPathComponents

// Go: checker/nodebuilderimpl.go:1081 sortedSymbolNamePair

// Go: checker/nodebuilderimpl.go:1858 SignatureToSignatureDeclarationOptions
#[derive(Default)]
pub struct SignatureToSignatureDeclarationOptions {
    pub modifiers: Vec<Node>,
    pub name: Node,
    pub question_token: Node,
}

// Go: checker/nodebuilderimpl.go:1172 canHaveModuleSpecifier

// Go: checker/nodebuilderimpl.go:1193 TryGetModuleSpecifierFromDeclaration
// Go: checker/nodebuilderimpl.go:1201 tryGetModuleSpecifierFromDeclarationWorker
// PORT: both are already ported as `try_get_module_specifier_from_declaration`
// in checker/extras.rs. This file calls that function.

// Go: checker/nodebuilderimpl.go:2234 hasTypeAnnotation

