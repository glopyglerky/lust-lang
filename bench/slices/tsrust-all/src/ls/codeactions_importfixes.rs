use crate::ls::prelude::*;

// Port of Go `ls/codeactions_importfixes.go`.
//
// PORT (whole file):
// - Go `program.GetTypeChecker(ctx)` is `ls_program::get_type_checker`.
//   Go leases the checker again in nested calls; the pool returns the
//   checker the request already holds. A second `RefCell` borrow would
//   panic, so a nested helper takes the caller's `ch: &mut Checker` (as the
//   w3 autoimport `View` methods do), and a lease that Go only hands to
//   `NewImportAdder` is held without a borrow (pinned decision: the adder
//   does not store the checker).
// - Go `*autoimport.View` is `Rc<autoimport::View>`; Go `*autoimport.Fix` is
//   `Rc<autoimport::Fix>`.
// - Go `[]*fixInfo` is `Vec<FixInfo>`; the values are never changed after
//   they are made, so a copied pointer is a clone.

use crate::frontend::core_ls_ext::compare_booleans;
use std::sync::LazyLock;

// Go: ls/codeactions_importfixes.go:20 importFixErrorCodes
// PORT: `diagnostics.X.Code()` is `int32`; `Message::code` is `u32`.

// Go: ls/codeactions_importfixes.go:45 importFixID
const IMPORT_FIX_ID: &str = "fixMissingImport";

// Go: ls/codeactions_importfixes.go:49 ImportFixProvider
// ImportFixProvider is the CodeFixProvider for import-related fixes

// Go: ls/codeactions_importfixes.go:56 fixInfo
#[derive(Clone)]
struct FixInfo {
    fix: Rc<autoimport::Fix>,
    symbol_name: String,
    error_identifier_text: String,
    is_jsx_namespace_fix: bool,
}

// Go: ls/codeactions_importfixes.go:63 getImportCodeActions

// Go: ls/codeactions_importfixes.go:95 getAllImportCodeActions

// Go: ls/codeactions_importfixes.go:152 addImportFromDiagnostic
// addImportFromDiagnostic finds the best import fix for a diagnostic and adds it to the adder.

// Go: ls/codeactions_importfixes.go:171 getFixInfos

// Go: ls/codeactions_importfixes.go:244 getFixesInfoForUMDImport

// Go: ls/codeactions_importfixes.go:268 getUmdSymbol

// Go: ls/codeactions_importfixes.go:297 isUMDExportSymbol
// PORT: Go reads the symbol through its pointer; here through the arena.

// Go: ls/codeactions_importfixes.go:303 getFixesInfoForNonUMDImport

// Go: ls/codeactions_importfixes.go:354 getTypeOnlyPromotionFix

// Go: ls/codeactions_importfixes.go:375 symbolNameInfo
struct SymbolNameInfo {
    name: String,
    is_type_only: bool, // whether the symbol currently resolves to a type-only import
}

// Go: ls/codeactions_importfixes.go:380 getSymbolNamesToImport

// Go: ls/codeactions_importfixes.go:411 needsJsxNamespaceFix

// Go: ls/codeactions_importfixes.go:425 jsxModeNeedsExplicitImport
fn jsx_mode_needs_explicit_import(jsx: JsxEmit) -> bool {
    jsx == JsxEmit::REACT || jsx == JsxEmit::REACT_NATIVE
}

// Go: ls/codeactions_importfixes.go:429 sortFixInfo
