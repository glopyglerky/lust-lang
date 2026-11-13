//! Port of typescript-go `internal/parser/parser.go` lines 1409-2826:
//! statements (switch, throw, try, labeled), variable declarations, binding
//! patterns, functions, classes and class members, interfaces, type aliases,
//! enums, modules, imports, exports and the start of type parsing.
//!
//! PORT: all `factory.NewX` calls hoist their arguments into locals first.
//! `self.factory` is borrowed immutably by the call, and a nested
//! `self.parse_x()` needs `&mut self`. Go evaluates the arguments left to
//! right before the call, so the order of parse calls is the same.
//!
//! PORT: Go `*diagnostics.Message` parameters that can be nil are
//! `Option<&'static Message>`. Go `nodeSliceArena` allocations are plain
//! slices passed to `new_node_list` / `new_modifier_list`.

use crate::frontend::prelude::*;


// Go: parser/parser.go:1816 isExportModifier

// Go: parser/parser.go:1820 isAsyncModifier


// Go: parser/parser.go:1847 isTypeHeritageClause
pub fn is_type_heritage_clause(is_interface: bool, token: SyntaxKind) -> bool {
    is_interface && token == SyntaxKind::ExtendsKeyword
        || !is_interface && token == SyntaxKind::ImplementsKeyword
}


// Go: parser/parser.go:1862 isValidHeritageTypeReferenceExpression


// Go: parser/parser.go:2011 modifierListHasAsync


// Go: parser/parser.go:2113 getSpaceSuggestion

