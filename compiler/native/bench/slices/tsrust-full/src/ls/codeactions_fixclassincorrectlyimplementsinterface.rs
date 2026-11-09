use crate::ls::prelude::*;

// Port of Go `ls/codeactions_fixclassincorrectlyimplementsinterface.go`.
//
// PORT (whole file):
// - Go `GetTypeCheckerForFile` + `defer done()` is
//   `ls_program::get_type_checker_for_file` with the `Release` guard kept to
//   the end of the function. The checker is borrowed only around the code
//   that uses it, because `getAllDiagnostics` and the import adder can lease
//   the same checker (a second `borrow_mut` would panic).
// - Go `autoimport.ImportAdder` (nil allowed) is
//   `Option<Box<dyn autoimport::ImportAdder>>` (w3 shape); helpers borrow it
//   as `Option<&mut dyn autoimport::ImportAdder>`.

use std::sync::LazyLock;

// Go: ls/codeactions_fixclassincorrectlyimplementsinterface.go:19 fixClassIncorrectlyImplementsInterfaceFixID
const FIX_CLASS_INCORRECTLY_IMPLEMENTS_INTERFACE_FIX_ID: &str =
    "fixClassIncorrectlyImplementsInterface";

// Go: ls/codeactions_fixclassincorrectlyimplementsinterface.go:21 fixClassIncorrectlyImplementsInterfaceErrorCodes

// Go: ls/codeactions_fixclassincorrectlyimplementsinterface.go:26 FixClassIncorrectlyImplementsInterfaceProvider

// Go: ls/codeactions_fixclassincorrectlyimplementsinterface.go:33 getCodeActionsToFixClassIncorrectlyImplementsInterface

// Go: ls/codeactions_fixclassincorrectlyimplementsinterface.go:69 getAllCodeActionsToFixClassIncorrectlyImplementsInterface

// Go: ls/codeactions_fixclassincorrectlyimplementsinterface.go:107 addChanges
// PORT: the fixer borrows `change_tracker` and `type_checker` for its whole
// life, so the Go uses of `changeTracker` and `typeChecker` next to it go
// through the fixer's fields.

// Go: ls/codeactions_fixclassincorrectlyimplementsinterface.go:136 getChanges

// Go: ls/codeactions_fixclassincorrectlyimplementsinterface.go:148 insertInterfaceMemberNode

// Go: ls/codeactions_fixclassincorrectlyimplementsinterface.go:156 getClass

// Go: ls/codeactions_fixclassincorrectlyimplementsinterface.go:164 getConstructor

// Go: ls/codeactions_fixclassincorrectlyimplementsinterface.go:176 getMissingMembers

// Go: ls/codeactions_fixclassincorrectlyimplementsinterface.go:207 getInheritedMembers
// PORT: Go returns a new `ast.SymbolTable` that only `getMissingMembers`
// reads by name. A local `FxHashMap<String, SymbolId>` stands for it, so no
// table is added to the checker's symbol arena.

// Go: ls/codeactions_fixclassincorrectlyimplementsinterface.go:231 createImportAdder
// PORT: Go also takes `typeChecker` and passes it to `NewImportAdder`. The
// pinned w3 `new_import_adder` has no checker parameter, so this drops it.
