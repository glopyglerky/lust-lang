//! Port of Go `ls/importTracker.go`.

use crate::ls::prelude::*;

// PORT (whole file):
// - Go `*checker.Checker` parameters are `&mut Checker`. The `ImportTracker`
//   closure takes the checker as its first argument instead of capturing it
//   (map-ls-navigation 2.3), and borrows the source file list and set for
//   the lifetime `'a` of the search state.
// - Go closures that share local state (`getImportersForExport`,
//   `getSearchesFromDirectImports`, `getImportOrExportSymbol`) become a small
//   local struct with methods, or nested functions with explicit parameters.
//   The Go call order is kept.
// - Go `[]*ast.SourceFile` is `&[Node]` (file roots). The source file name
//   set (`*collections.Set[string]`) is `&FxHashSet<String>`.
// - Go `*ExportInfo` that can be nil is `Option<ExportInfo>`; a non-nil
//   parameter is `&ExportInfo`.

use crate::flags_macros::go_enum;
use crate::frontend::tspath;
use std::rc::Weak;

// Go: ls/importTracker.go:15 ImpExpKind
go_enum!(ImpExpKind, i32 {
    UNKNOWN = 0;
    IMPORT = 1;
    EXPORT = 2;
});

// Go: ls/importTracker.go:23 ImportExportSymbol
#[derive(Clone, Copy, Debug)]
pub struct ImportExportSymbol {
    pub kind: ImpExpKind,
    pub symbol: SymbolId,
    pub export_info: Option<ExportInfo>,
}

// Go: ls/importTracker.go:29 ExportKind
go_enum!(ExportKind, i32 {
    NAMED = 0;
    DEFAULT = 1;
    EXPORT_EQUALS = 2;
    UMD = 3;
    MODULE = 4;
});

// Go: ls/importTracker.go:39 ExportInfo
#[derive(Clone, Copy, Debug)]
pub struct ExportInfo {
    pub exporting_module_symbol: SymbolId,
    pub export_kind: ExportKind,
}

// Go: ls/importTracker.go:44 LocationAndSymbol
#[derive(Clone, Copy, Debug)]
pub struct LocationAndSymbol {
    pub import_location: Node,
    pub import_symbol: SymbolId,
}

// Go: ls/importTracker.go:49 ImportsResult
// PORT: Go `indirectUsers []*ast.SourceFile` holds file roots.
#[derive(Clone, Debug, Default)]
pub struct ImportsResult {
    pub import_searches: Vec<LocationAndSymbol>,
    pub single_references: Vec<Node>,
    pub indirect_users: Vec<Node>,
}

// Go: ls/importTracker.go:55 ImportTracker
// PORT: the checker is the first argument (Go captures it). Go returns
// `*ImportsResult`; the result is returned by value. `'a` is the lifetime of
// the source file list and set that the tracker borrows.

// Go: ls/importTracker.go:57 ModuleReferenceKind
go_enum!(ModuleReferenceKind, i32 {
    IMPORT = 0;
    REFERENCE = 1;
    IMPLICIT = 2;
});

// Go: ls/importTracker.go:66 ModuleReference
// ModuleReference represents a reference to a module, either via import, <reference>, or implicit reference
// PORT: Go `ref *ast.FileReference` is a copy of the reference, None for nil.
#[derive(Clone, Debug)]
pub struct ModuleReference {
    pub kind: ModuleReferenceKind,
    pub literal: Node, // for import and implicit kinds (StringLiteralLike)
    pub referencing_file: Node,
    pub ref_: Option<FileReference>, // for reference kind
}

// Go: ls/importTracker.go:74 createImportTracker
// Creates the imports map and returns an ImportTracker that uses it. Call this lazily to avoid calling `getDirectImportsMap` unnecessarily.
// PORT: the returned closure borrows `sourceFiles` and `sourceFilesSet`
// (Go captures the caller's slice and set, which do not change during a
// search). It takes the checker as its first argument.
// PERF: the map comes from `get_direct_imports_map_cached`, which reuses the
// map of an earlier search with the same checker.

// Go: ls/importTracker.go:84 getDirectImportsMap
// Returns a map from a module symbol to all import statements that directly reference the module




// Go: ls/importTracker.go:100 forEachImport
// Calls `action` for each import, re-export, or require() in a file
// PORT: Go `sourceFile.Path()` is the installed program's path of the file
// (`source_file_info(file).path`).

// Go: ls/importTracker.go:134 forEachPossibleImportOrExportStatement

// Go: ls/importTracker.go:143 getSourceFileLikeForImportDeclaration

// Go: ls/importTracker.go:155 isAmbientModuleDeclaration

// Go: ls/importTracker.go:159 getStatementsOfSourceFileLike

// Go: ls/importTracker.go:169 getImportersForExport
// PORT: the Go closures share `directImports`, `indirectUserDeclarations`
// and the two seen trackers. They are the methods of `ImportersForExport`,
// in Go order.



// Go: ls/importTracker.go:213 isExported (closure in getImportersForExport)
// PORT: the closure reads no shared state, so it is a free function. The
// name carries the enclosing function to keep it apart from other
// `is_exported` names.

// Go: ls/importTracker.go:324 getContainingModuleSymbol

// Go: ls/importTracker.go:329 findNamespaceReExports
// Returns 'true' is the namespace 'name' is re-exported from this module, and 'false' if it is only used locally

// Go: ls/importTracker.go:343 getSearchesFromDirectImports
// PORT: the Go closures share `importSearches` and `singleReferences`. They
// are the methods of `SearchesFromDirectImports`, in Go order.



// Go: ls/importTracker.go:462 getImportOrExportSymbol
// PORT: the Go closures `exportInfo`, `getExport` (with its nested
// closures) and `getImport` are nested functions that take the captured
// values as parameters.

// Go: ls/importTracker.go:611 getExportInfo

// Go: ls/importTracker.go:628 getExportNode
// If a reference is a class expression, the exported node would be its parent.
// If a reference is a variable declaration, the exported node would be the variable statement.

// Go: ls/importTracker.go:645 isNodeImport

// Go: ls/importTracker.go:662 isExternalModuleImportEquals

// Go: ls/importTracker.go:668 skipExportSpecifierSymbol
// If at an export specifier, go to the symbol it refers to. */

// Go: ls/importTracker.go:684 getExportEqualsLocalSymbol

// Go: ls/importTracker.go:701 symbolNameNoDefault

// Go: ls/importTracker.go:716 findModuleReferences
// findModuleReferences finds all references to a module symbol across the given source files.
// This includes import statements, <reference> directives, and implicit references (e.g., JSX runtime imports).
// PORT: the `<reference path>` and `<reference types>` checks read the
// program's parsed file; they are `ProgramView::references_to_file`
// (program_view.rs), in the same order.
