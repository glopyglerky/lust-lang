use crate::ls::autoimport::prelude::*;

// Port of Go `ls/autoimport/extract.go`.
//
// PORT: Go keeps `checker *checker.Checker` in the extractors. A Rust
// struct holds it as `&'c mut Checker` for the extractor's life (an
// extractor lives for one file or one symbol). Symbols are read from that
// checker's arena (`checker.symbols`), which holds every bound symbol plus
// the checker's own symbols, as the Go pointers do.
// PORT: Go map iteration over symbol tables (`file.Symbol.Exports`,
// `target.Exports`, ...) has random order; the port uses table insertion
// order. Export order only decides the order of index entries (ties in the
// completion sort).

use crate::frontend::module;
use crate::frontend::tspath;
use crate::ls::lsutil;
use std::cell::Cell;

// Go: ls/autoimport/extract.go:16 symbolExtractor

// Go: ls/autoimport/extract.go:29 exportExtractor
// PORT: Go embeds `*symbolExtractor`; it is the nested `symbol_extractor`
// field, and `Deref` promotes its fields and methods as Go does.



// Go: ls/autoimport/extract.go:34 extractorStats
// PORT: Go `atomic.Int32` counters are `Cell<i32>` (one thread).
#[derive(Debug, Default)]
pub struct ExtractorStats {
    pub exports: Cell<i32>,
    pub used_checker: Cell<i32>,
}


// Go: ls/autoimport/extract.go:43 checkerLease
// PORT: Go keeps a second pointer to the extractor's checker. Rust cannot
// hold a second `&mut Checker`, so the lease keeps only `used`, and its
// methods take the extractor's checker and give it back.
#[derive(Clone, Copy, Debug, Default)]
pub struct CheckerLease {
    pub used: bool,
}

impl CheckerLease {
    // Go: ls/autoimport/extract.go:48 GetChecker

    // Go: ls/autoimport/extract.go:53 TryChecker

}

// Go: ls/autoimport/extract.go:60 newSymbolExtractor
// PORT: Go nil funcs are `None`. Go `&binder.NameResolver{CompilerOptions:
// core.EmptyCompilerOptions}` leaves every callback nil.




// Go: ls/autoimport/extract.go:128 isNonPatternAmbientModuleDeclaration
// Reports whether `decl` is not a pattern ambient module (`declare module
// "*.css"`). Auto-import skips pattern modules: no import can name them.



// Go: ls/autoimport/extract.go:410 shouldIgnoreSymbol
// PORT: the arena parameter as in `try_get_module_id_and_file_name_of_module_symbol`.

// Go: ls/autoimport/extract.go:417 getSyntax
// PORT: the arena parameter as in `try_get_module_id_and_file_name_of_module_symbol`.

// Go: ls/autoimport/extract.go:448 isUnusableName
pub fn is_unusable_name(name: &str) -> bool {
    name.is_empty()
        || name == "_default"
        || name == INTERNAL_SYMBOL_NAME_EXPORT_STAR
        || name == INTERNAL_SYMBOL_NAME_DEFAULT
        || name == INTERNAL_SYMBOL_NAME_EXPORT_EQUALS
}

// Go: ls/autoimport/extract.go:461 fileNameForDefaultExportName
// fileNameForDefaultExportName returns the best file name to use when deriving
// a fallback identifier for a default-like export. It prefers the target symbol's
// source file (closest to the export origin), falls back to the module's original
// file name, and uses the lowercased moduleID only for ambient modules where no
// original file name is available.
// PORT: the arena parameter as in `try_get_module_id_and_file_name_of_module_symbol`.
