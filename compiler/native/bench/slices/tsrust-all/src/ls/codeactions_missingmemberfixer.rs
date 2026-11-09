use crate::ls::prelude::*;

// Port of Go `ls/codeactions_missingmemberfixer.go`.
//
// PORT (whole file):
// - Go `missingMemberFixer` holds pointers to the change tracker, the
//   checker and the import adder, and its caller keeps using all three. The
//   Rust fixer borrows them (`&'a mut`); the caller reads them back through
//   the pub fields while the fixer lives.
// - Go `autoimport.ImportAdder` is an interface value that can be nil. The
//   caller owns an `Option<Box<dyn autoimport::ImportAdder>>` (w3 shape) and
//   the fixer borrows it as `Option<&mut dyn autoimport::ImportAdder>`.
// - Go `NewNodeBuilderEx` shares its `idToSymbol` map with the caller. The
//   Rust builder owns the map; the fixer reads it back from
//   `nb.impl_.borrow().id_to_symbol` after each build (PORTING "Programs and
//   checkers").
// - Go reads `*ast.Symbol` fields directly. Here they are read from the
//   checker's symbol arena (`type_checker.sym(s)`).

use crate::flags_macros::go_flags;

// Go: ls/codeactions_missingmemberfixer.go:18 preserveOptionalFlags
go_flags!(PreserveOptionalFlags, i32 {
    METHOD = 1 << 0; // preserveOptionalFlagsMethod
    PROPERTY = 1 << 1; // preserveOptionalFlagsProperty
    ALL = (1 << 0) | (1 << 1); // preserveOptionalFlagsAll
});

// Go: ls/codeactions_missingmemberfixer.go:26 missingMemberFixer

// Go: ls/codeactions_missingmemberfixer.go:35 newMissingMemberFixer



// Go: ls/codeactions_missingmemberfixer.go:451 createDummyParameters

// Go: ls/codeactions_missingmemberfixer.go:489 createDeclarationName

// Go: ls/codeactions_missingmemberfixer.go:505 createPropertyName
