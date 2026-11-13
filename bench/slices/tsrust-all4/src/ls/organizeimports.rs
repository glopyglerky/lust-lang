use crate::ls::prelude::*;

// Port of Go `ls/organizeimports.go`.
//
// PORT (whole file):
// - Go `[]*ast.Statement` is `Vec<Node>` (param `&[Node]`); `[][]*ast.Statement`
//   is `Vec<Vec<Node>>`.
// - Go `func(a, b string) int` comparers are `lsutil::StringComparer`
//   (`Option` where Go can hold nil); `func(s1, s2 *ast.Node) int` is
//   `lsutil::NodeComparer`.
// - Go `ast.NewNodeFactory(ast.NodeFactoryHooks{})` is `NodeFactory::new()`
//   (no hooks).
// - Go `changeTracker.SetEmitFlags` / `AddEmitFlags` (promoted from the
//   embedded `*printer.EmitContext`) are `change_tracker.emit_context.x(..)`.
// - Go `slices.SortFunc` is `gostd::slices::sort_func` (Go pdqsort);
//   `slices.SortStableFunc` is `gostd::slices::sort_stable_func`.

use crate::astdata::NodeData;
use crate::frontend::scanner::{};
use crate::frontend::stringutil_ls;


// Go: ls/organizeimports.go:120 organizeImportsComparerSettings
// PORT: a nil Go comparer func is `None`.
#[derive(Clone)]
struct OrganizeImportsComparerSettings {
    module_specifier_comparer: Option<lsutil::StringComparer>,
    named_import_comparer: Option<lsutil::StringComparer>,
    type_order: lsutil::OrganizeImportsTypeOrder,
}


// Go: ls/organizeimports.go:126 organizeImportsWorker

// Go: ls/organizeimports.go:220 groupByModuleSpecifier

// Go: ls/organizeimports.go:239 removeUnusedImports

// Go: ls/organizeimports.go:320 filterUsedImportSpecifiers

// Go: ls/organizeimports.go:337 hasModuleDeclarationMatchingSpecifier

// Go: ls/organizeimports.go:353 getImportAttributesKey
// getImportAttributesKey returns a key for grouping imports by their attributes.

// Go: ls/organizeimports.go:389 groupByNewlineContiguous
// groupByNewlineContiguous groups declarations by blank lines between them.

// Go: ls/organizeimports.go:410 isNewGroup
// PORT: Go takes the source file; the caller passes its text, which the
// scanner borrows.

// Go: ls/organizeimports.go:445 coalesceImportsWorker

// Go: ls/organizeimports.go:635 categorizedImports
struct CategorizedImports {
    import_without_clause: Node,
    type_only_imports: ImportGroup,
    regular_imports: ImportGroup,
}

// Go: ls/organizeimports.go:641 importGroup
#[derive(Clone, Default)]
struct ImportGroup {
    default_imports: Vec<Node>,
    namespace_imports: Vec<Node>,
    named_imports: Vec<Node>,
}

impl ImportGroup {
    // Go: ls/organizeimports.go:643 (importGroup).isEmpty
    fn is_empty(&self) -> bool {
        self.default_imports.is_empty()
            && self.namespace_imports.is_empty()
            && self.named_imports.is_empty()
    }
}

// Go: ls/organizeimports.go:651 getCategorizedImports

// Go: ls/organizeimports.go:693 getNewImportSpecifiers

// Go: ls/organizeimports.go:723 tryGetNamedBindingElements
// PORT: a Go nil slice is `None`.

// Go: ls/organizeimports.go:744 getTopLevelExportGroups

// Go: ls/organizeimports.go:784 organizeExportsWorker

// Go: ls/organizeimports.go:833 coalesceExportsWorker

// Go: ls/organizeimports.go:929 categorizedExports
struct CategorizedExports {
    export_without_clause: Node,
    named_exports: Vec<Node>,
    type_only_exports: Vec<Node>,
}

// Go: ls/organizeimports.go:935 getCategorizedExports

// Go: ast/ast.go:273 (*Node).Text
