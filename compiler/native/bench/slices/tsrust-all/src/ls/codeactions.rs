use crate::ls::prelude::*;

use crate::spanmap::Feature;

// Port of Go `ls/codeactions.go`.
//
// PORT (whole file):
// - Go `*CodeFixProvider` values are package vars; here they are
//   `pub static X: LazyLock<CodeFixProvider>` and a provider pointer is
//   `&'static CodeFixProvider` (pointer identity is `std::ptr::eq`).
// - Go `*CodeFixContext` borrows the language service, the diagnostic and
//   the params for one call, so it is `CodeFixContext<'a>` with references.
// - Go `[]*CodeAction` holds pointers that are never changed after they are
//   made; here `Vec<CodeAction>` values (cloned where Go shares a pointer).
// - Go `map[...]` values that reach output use `IndexMap` in insertion
//   order. Go map order is random; the oracle compares those outputs
//   without order.

// Go: ls/codeactions.go:21 CodeFixProvider
// CodeFixProvider represents a provider for a specific type of code fix
// PORT: Go func fields are plain `fn` pointers, so the statics are `Sync`.
// A nil Go `GetAllCodeActions` is `None`.

// Go: ls/codeactions.go:29 CodeFixContext
// CodeFixContext contains the context needed to generate code fixes
// PORT: Go nil `Diagnostic` and `Params` are `None`. Go leaves `Span` and
// `ErrorCode` at their zero values in the fix-all contexts; Rust callers
// write `TextRange::default()` and `0`.

// Go: ls/codeactions.go:40 CodeAction
// CodeAction represents a single code action fix
#[derive(Clone, Debug, Default)]
pub struct CodeAction {
    pub description: String,
    pub changes: Vec<lsproto::TextEdit>,
    pub fix_id: String,
    pub fix_all_description: String,
}

impl CodeAction {
    // Go: ls/codeactions.go:48 (*CodeAction).Compare
    // Compare defines a total ordering for CodeAction values, comparing description
    // then text edits lexicographically. Used with slices.BinarySearchFunc.
    // PORT: Go `strings.Compare` is a byte-wise comparison, like `str::cmp`.
    pub fn compare(&self, b: &CodeAction) -> i32 {
        let a = self;
        let c = a.description.as_str().cmp(b.description.as_str()) as i32;
        if c != 0 {
            return c;
        }
        let c = a.changes.len().cmp(&b.changes.len()) as i32;
        if c != 0 {
            return c;
        }
        for (i, edit) in a.changes.iter().enumerate() {
            let c = edit.compare(&b.changes[i]);
            if c != 0 {
                return c;
            }
        }
        0
    }
}

// Go: ls/codeactions.go:65 CombinedCodeActions
// CombinedCodeActions represents combined code actions for fix-all scenarios
#[derive(Clone, Debug, Default)]
pub struct CombinedCodeActions {
    pub description: String,
    pub changes: Vec<lsproto::TextEdit>,
}

// Go: ls/codeactions.go:71 codeFixProviders
// codeFixProviders is the list of all registered code fix providers
// PORT: a Go package var; the list is rebuilt from the provider statics on
// each call, in Go order.


// Go: ls/codeactions.go:218 hasMultipleFixableDiagnostics
// hasMultipleFixableDiagnostics returns true if the file has at least 2 diagnostics
// matching the given error codes. Checks all diagnostic sources (semantic,
// syntactic, suggestion, declaration) to match ProvideDiagnostics.

// Go: ls/codeactions.go:232 codeFixProviderMatchesLSPDiagnostic

// Go: ls/codeactions.go:239 isFixableDiagnostic

// Go: ls/codeactions.go:244 isFixAllKind
// isFixAllKind returns true if the requested kind matches source.fixAll
fn is_fix_all_kind(kind: &lsproto::CodeActionKind) -> bool {
    kind.contains(&lsproto::CodeActionKind::SOURCE_FIX_ALL_TS)
}

// Go: ls/codeactions.go:250 wantsQuickFixes
// wantsQuickFixes returns true if the Only filter is nil/empty (meaning all kinds are wanted)
// or explicitly includes the quickfix kind.
// PORT: Go `*[]lsproto.CodeActionKind`; nil is `None`.
fn wants_quick_fixes(only: Option<&[lsproto::CodeActionKind]>) -> bool {
    let Some(only) = only else {
        return true;
    };
    if only.is_empty() {
        return true;
    }
    for kind in only {
        if kind.contains(&lsproto::CodeActionKind::QUICK_FIX) {
            return true;
        }
    }
    false
}


// Go: ls/codeactions.go:307 getOrganizeImportsActionTitle
// getOrganizeImportsActionTitle returns the appropriate title for the given organize imports kind

// Go: ls/codeactions.go:321 getOrganizeImportsActionsForKind
// getOrganizeImportsActionsForKind returns the organize imports code action kinds that should be
// returned for the given requested kind.
fn get_organize_imports_actions_for_kind(
    requested_kind: &lsproto::CodeActionKind,
) -> Vec<lsproto::CodeActionKind> {
    let organize_imports_kinds = [
        lsproto::CodeActionKind::SOURCE_ORGANIZE_IMPORTS_TS,
        lsproto::CodeActionKind::SOURCE_REMOVE_UNUSED_IMPORTS_TS,
        lsproto::CodeActionKind::SOURCE_SORT_IMPORTS_TS,
    ];

    let mut result: Vec<lsproto::CodeActionKind> = Vec::new();
    for organize_kind in organize_imports_kinds {
        if requested_kind.contains(&organize_kind) {
            result.push(organize_kind);
        }
    }

    if result.contains(requested_kind) {
        return vec![requested_kind.clone()];
    }

    result
}


// Go: ls/codeactions.go:382 containsErrorCode
// containsErrorCode checks if the error code is in the list
pub fn contains_error_code(codes: &[i32], code: i32) -> bool {
    codes.contains(&code)
}

// Go: ls/codeactions.go:387 convertToLSPCodeAction
// convertToLSPCodeAction converts an internal CodeAction to an LSP CodeAction
