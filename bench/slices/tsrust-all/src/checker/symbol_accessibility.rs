//! Port of checker/symbolaccessibility.go.

use crate::prelude::*;
use crate::printer::{SymbolAccessibility, SymbolAccessibilityResult};


// Go: checker/symbolaccessibility.go:105 hasNonGlobalAugmentationExternalModuleSymbol

// Go: checker/symbolaccessibility.go:109 getQualifiedLeftMeaning
pub fn get_qualified_left_meaning(right_meaning: SymbolFlags) -> SymbolFlags {
    // If we are looking in value space, the parent meaning is value, other wise it is namespace
    if right_meaning == SymbolFlags::VALUE {
        return SymbolFlags::VALUE;
    }
    SymbolFlags::NAMESPACE
}

// Go: checker/symbolaccessibility.go:249 hasExternalModuleSymbol

// Go: checker/symbolaccessibility.go:391 accessibleSymbolChainContext
// PORT: Go shares `visitedSymbolTablesMap` by map reference between contexts.
// Here the map is an `Rc<RefCell<..>>` so clones share it the same way.
#[derive(Clone)]
pub(crate) struct AccessibleSymbolChainContext {
    pub symbol: SymbolId,
    pub enclosing_declaration: Node,
    pub meaning: SymbolFlags,
    pub use_only_external_aliasing: bool,
    pub visited_symbol_tables_map: Rc<RefCell<FxHashMap<SymbolId, FxHashSet<SymbolTableID>>>>,
}

// Go: checker/symbolaccessibility.go:402 symbolTableID constants
// The high 3 bits encode the kind, and the remaining bits encode the
// NodeId or SymbolId of the source.
impl SymbolTableID {
    const KIND_SHIFT: u32 = 61;


    // Go: checker/symbolaccessibility.go:417 symbolTableIDFromLocals

    // Go: checker/symbolaccessibility.go:421 symbolTableIDFromExports

    // Go: checker/symbolaccessibility.go:429 symbolTableIDFromResolvedExports

    // Go: checker/symbolaccessibility.go:433 symbolTableIDFromMembers

    // Go: checker/symbolaccessibility.go:437 symbolTableIDFromGlobals
}

// Go: checker/symbolaccessibility.go:612 isUMDExportSymbol

// Go: checker/symbolaccessibility.go:616 isNamespaceReexportDeclaration

// Go: checker/symbolaccessibility.go:728 isPropertyOrMethodDeclarationSymbol
