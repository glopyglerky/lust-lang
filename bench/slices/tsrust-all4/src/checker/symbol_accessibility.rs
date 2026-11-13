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
    pub const KIND_LOCALS: Self = Self(0 << Self::KIND_SHIFT);
    pub const KIND_EXPORTS: Self = Self(1 << Self::KIND_SHIFT);
    pub const KIND_MEMBERS: Self = Self(2 << Self::KIND_SHIFT);
    pub const KIND_GLOBALS: Self = Self(3 << Self::KIND_SHIFT);
    /// Resolved/derived exports from getExportsOfSymbol, distinct from raw sym.Exports.
    pub const KIND_RESOLVED_EXPORTS: Self = Self(4 << Self::KIND_SHIFT);
    /// Go `stKindMask = (iota - 1) << stKindShift`, with iota = 5.
    pub const KIND_MASK: Self = Self(4 << Self::KIND_SHIFT);

    /// Go `tableId & stKindMask`.
    #[must_use]
    pub fn kind(self) -> Self {
        Self(self.0 & Self::KIND_MASK.0)
    }

    // Go: checker/symbolaccessibility.go:417 symbolTableIDFromLocals

    // Go: checker/symbolaccessibility.go:421 symbolTableIDFromExports

    // Go: checker/symbolaccessibility.go:429 symbolTableIDFromResolvedExports

    // Go: checker/symbolaccessibility.go:433 symbolTableIDFromMembers

    // Go: checker/symbolaccessibility.go:437 symbolTableIDFromGlobals
    #[must_use]
    pub fn from_globals() -> Self {
        Self::KIND_GLOBALS
    }
}

// Go: checker/symbolaccessibility.go:612 isUMDExportSymbol

// Go: checker/symbolaccessibility.go:616 isNamespaceReexportDeclaration

// Go: checker/symbolaccessibility.go:728 isPropertyOrMethodDeclarationSymbol
