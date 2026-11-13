use crate::ls::autoimport::prelude::*;

// Port of Go `ls/autoimport/util.go`.

use crate::frontend::compiler;
use crate::frontend::module;
use crate::frontend::packagejson;
use crate::frontend::scanner::scanner_p1::{
    
};
use crate::frontend::tspath;
use crate::frontend::vfs;
use crate::gostd::unicode;
use crate::modulespecifiers;
use crate::program::ls_program;
use std::cell::Cell;
use std::collections::VecDeque;
use std::time::SystemTime;

// Go: ls/autoimport/util.go:24 tryGetModuleIDAndFileNameOfModuleSymbol
// PORT: Go reads the symbol through its pointer; the port takes the arena
// that owns it (a checker's `symbols`), like the ast helpers.

// Go: ls/autoimport/util.go:41 getModuleIDAndFileNameOfModuleSymbol
// PORT: the arena parameter as in `try_get_module_id_and_file_name_of_module_symbol`.

// Go: ls/autoimport/util.go:68 wordIndices
// wordIndices splits an identifier into its constituent words based on camelCase and snake_case conventions
// by returning the starting byte indices of each word. The first index is always 0.
//   - CamelCase
//     ^    ^
//   - snake_case
//     ^     ^
//   - ParseURL
//     ^    ^
//   - __proto__
//     ^
// PORT: Go ranges over the runes of `s`; `char_indices` gives the same byte
// indices. Go decodes `s[byteIndex+1:]` even inside a multi-byte rune; the
// scanner byte decoders do the same without slicing inside a character.

/// A decoded Go rune as a `char` (`utf8.RuneError` is U+FFFD).
fn rune_of(r: i32) -> char {
    char::from_u32(r as u32).unwrap_or(char::REPLACEMENT_CHARACTER)
}

// Go: ls/autoimport/util.go:88 getPackageNamesInNodeModules
// PORT: Go returns a non-nil `*collections.Set[string]`; the port returns the set.

// Go: ls/autoimport/util.go:118 getDefaultLikeExportNameFromDeclaration
// PORT: the arena parameter as in `try_get_module_id_and_file_name_of_module_symbol`.


// Go: ls/autoimport/util.go:145 getResolvedPackageNames
// PORT: Go returns a non-nil `*collections.Set[string]`; the port returns the
// set. Go ranges over map keys in random order; the port uses map order
// (the results go into a set).

// Go: ls/autoimport/util.go:183 addProjectReferenceOutputMappings
// addProjectReferenceOutputMappings adds output .d.ts to source file mappings
// from a program's project references to the provided map.
// This is used during node_modules bucket building to redirect extraction
// from output files to source files when the output is from a project reference.

// Go: ls/autoimport/util.go:199 createCheckerPool
// PORT: Go makes up to GOMAXPROCS new checkers with `checker.NewChecker(program,
// nil)` over any `checker.Program`. Here `program` is a `NewProgram` and each
// checker is `ls_program::new_checker(program)` (contract C3): a new checker
// on the dispatch thread, like Go. The Go channel is
// a FIFO queue; the atomic counter is a `Cell`. Extraction runs serially in
// Go start order, so a released checker is back in the pool before the next
// request and only one checker is made. Go blocks on an empty pool at the
// limit; serial code cannot wait for another goroutine, so that case panics
// with the Go deadlock text. A release after `closePool` panics like a send
// on a closed channel.

// Go: ls/autoimport/util.go:234 addPackageJsonDependencies
// addPackageJsonDependencies adds all dependencies and peerDependencies from a package.json
// to the given set, canonicalizing @types package names to their base names.

// Go: ls/autoimport/util.go:253 getPackageRealpathFuncs
// getPackageRealpathFuncs returns functions to transform between symlink and realpath for files within a package.
// It calls FS.Realpath once per package directory and uses prefix substitution for files within that directory,
// avoiding expensive realpath syscalls for each file. For files outside the package (e.g. re-exported
// dependencies reached through node_modules symlinks), it resolves the file's directory realpath once,
// finds the symlink boundary (the package root where the symlink lives), and caches that prefix mapping.
// All subsequent files under the same symlinked package directory use prefix substitution with no syscalls.
// PORT: Go `func(string) string` values are `Rc<dyn Fn(&str) -> String>`.

// Go: ls/autoimport/util.go:302 resolutionHost
// PORT: private like Go; the trait is `module::ResolutionHost`.

// Go: ls/autoimport/util.go:307 var _ module.ResolutionHost = (*resolutionHost)(nil)

// Go: ls/autoimport/util.go:317 getModuleResolver
// PORT: Go `core.EmptyCompilerOptions` is shared; the resolver takes an `Rc`,
// so it gets a new default `CompilerOptions`. Go `*module.DefaultResolver` is
// `Rc<module::DefaultResolver>`.


