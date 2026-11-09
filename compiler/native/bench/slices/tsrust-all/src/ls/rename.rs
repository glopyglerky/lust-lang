//! Port of Go `ls/rename.go`.
//!
//! PORT: Go `*compiler.Program` is `&compiler::NewProgram`. A Go
//! checker lease `ch, done := program.GetTypeChecker(ctx)` is
//! `ls_program::get_type_checker`, with `done` kept alive to the end of the
//! scope (Go `defer done()`).
//! PORT: `symbolAndEntriesToRename` and what it calls are generic over the
//! program (`ProgramView`, program_view.rs) so that a cross-project rename
//! can run them on a search thread; there the lease is
//! `ProgramView::get_type_checker`.

use crate::ls::prelude::*;

use crate::spanmap::Feature;

// Go: ls/rename.go:25 RenameInfo
// RenameInfo represents the result of a rename validation check.
// It is used by the `textDocument/prepareRename` LSP handler.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenameInfo {
    pub can_rename: bool,
    pub localized_error_message: String,
    pub display_name: String,
    pub trigger_span: lsproto::Range,
    pub file_to_rename: String,
    pub new_file_name: String,
}

// Go: ls/rename.go:34 mappedRenameEdit
struct MappedRenameEdit {
    uri: lsproto::DocumentUri,
    edit: lsproto::TextEdit,
}

// Go: ls/rename.go:39 renameEditKey
#[derive(PartialEq, Eq, Hash)]
struct RenameEditKey {
    uri: lsproto::DocumentUri,
    text_range: lsproto::Range,
}

// Go: ls/rename.go:44 deduplicateRenameEdits
// PORT: Go returns `(nil, false)` on a conflict; here `None`. Go map order
// is random, so the IndexMap keeps first-insert order, as the old
// `changes` map of `symbolAndEntriesToRename` did.



/// `ProvideRename` as a `CrossProjectSearch`: its searches in other projects
/// run on search threads.
pub struct RenameSearch;


// Go: ls/rename.go:209 nodeIsEligibleForRename


// Go: ls/rename.go:249 isDefinedInLibraryFile
// isDefinedInLibraryFile checks if a declaration is from a default library file (e.g., lib.d.ts).

// Go: ls/rename.go:255 wouldRenameInOtherNodeModules
// wouldRenameInOtherNodeModules checks if renaming the symbol would affect node_modules.

// Go: ls/rename.go:290 ClientSupportsWillRenameFiles

// Go: ls/rename.go:294 ClientSupportsDocumentChanges

// Go: ls/rename.go:298 ClientSupportsRenameResourceOperations


// Go: ls/rename.go:423 getQuoteFromPreference
pub fn get_quote_from_preference(quote_preference: lsutil::QuotePreference) -> &'static str {
    if quote_preference == lsutil::QuotePreference::SINGLE {
        return "'";
    }
    "\""
}

// Go: ls/rename.go:430 getRenameInfoError

// Go: ls/rename.go:437 getRenameInfoSuccess
