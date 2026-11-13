//! Small Go functions that live outside the ported ranges (emitresolver.go,
//! nodebuilderimpl.go, symbolaccessibility.go) but that ported checker code
//! calls. Also the checker's synthetic flow nodes.

use crate::prelude::*;

/// File index for flow nodes that the checker creates. Go allocates
/// `&ast.FlowNode{}` values; here they live in a leaked thread-local list.
pub const SYNTHETIC_FLOW_FILE: usize = 0xffff_ffff;

thread_local! {
    static SYNTHETIC_FLOWS: RefCell<Vec<&'static FlowNode>> = const { RefCell::new(Vec::new()) };
}



// Go: checker/nodebuilderimpl.go:1193 TryGetModuleSpecifierFromDeclaration

// Go: checker/nodebuilderimpl.go:1201 tryGetModuleSpecifierFromDeclarationWorker
