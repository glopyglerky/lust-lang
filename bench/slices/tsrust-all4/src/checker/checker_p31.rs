//! Port of `checker/checker.go` lines 27805-28752: indexed access and
//! conditional type simplification, normalization helpers, regular object
//! literal types, alias reference marking (linked references, JSX, decorator
//! metadata), external emit helpers, and the promised type of a promise.

use crate::prelude::*;


// Go: checker/checker.go:28821 isExportOrExportExpression

// Go: checker/checker.go:28836 shouldMarkIdentifierAliasReferenced
// PERF: chkA. The parent and the great-grandparent come with their kinds
// (`node_parent_and_kind`, one store lookup each), so each node is looked up
// once. The tests and their order are Go's.

// Go: checker/checker.go:28857 isInternalModuleImportEqualsDeclaration
// PORT: `ast.IsInternalModuleImportEqualsDeclaration` already ports to the
// free fn `is_internal_module_import_equals_declaration` (ast/utilities).
// This checker-package copy is private and suffixed so the prelude globs do
// not see two items with one name.


// Go: checker/checker.go:28944 isPartOfImportEqualsModuleReference


// Go: checker/checker.go:29357 getEntityNameFromTypeNode

