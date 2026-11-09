//! Port of Go `ls/file_rename.go`.
//!
//! PORT: Go `*compiler.Program` is `&compiler::NewProgram`. Go
//! `*change.Tracker` is `&mut change::Tracker`. Go passes an
//! `*ast.SourceFile` where a program method takes an `ast.HasFileName`; here
//! that is `autoimport::source_file_has_file_name(file)`.

use crate::ls::prelude::*;

// Go: ls/file_rename.go:22 pathUpdater
// PORT: a Go func type. `createPathUpdater` returns it boxed; other functions
// take it as `&PathUpdater`.
pub type PathUpdater<'a> = dyn Fn(&str) -> (String, bool) + 'a;

// Go: ls/file_rename.go:24 toImport
#[derive(Clone, Debug, Default)]
pub struct ToImport {
    pub new_file_name: String,
    pub updated: bool,
}

// Go: ls/file_rename.go:29 movedFile
#[derive(Clone, Debug, Default)]
pub struct MovedFile {
    pub source_file: Node,
    pub new_file_name: String,
}


// Go: ls/file_rename.go:177 updatePathsProperty

// Go: ls/file_rename.go:190 tryUpdateConfigString

// Go: ls/file_rename.go:89 createPathUpdater (the body, see
// `LanguageService::create_path_updater`)


// Go: ls/file_rename.go:308 getSourceFileToImport
// PORT: Go returns `*toImport`; nil is `None`.

// Go: ls/file_rename.go:327 getUpdatedImportSpecifierFromMovedSourceFiles
// As a fall back for unresolved modules, we'll check every file affected by the rename to see if any of them would match
// the import specifier, and if so, we'll obtain the updated specifier for that file.
// PORT: Go passes the program as the `ModuleSpecifierGenerationHost`; here
// that is `modulespecifiers::ProgramHost` (the installed program).

// Go: ls/file_rename.go:362 createStringTextRange
// PORT: Go `End()-1` steps back one Go byte (`go_offset_before`), as in
// `try_update_config_string`.

// Go: ls/file_rename.go:366 getTsConfigObjectLiteralExpression
// PORT: Go returns `*ast.ObjectLiteralExpression`; nil is `Node::NIL`.

// Go: ls/file_rename.go:376 forEachObjectProperty
// PORT: Go `cb func(property *ast.PropertyAssignment, propertyName string)`.

// Go: ls/file_rename.go:390 relativePathFromDirectory

// Go: ls/file_rename.go:394 relativeImportPathFromDirectory

// Go: ls/file_rename.go:398 isAmbientModuleSymbol
// PORT: Go reads `symbol.Declarations` without a checker; the symbol arena
// is the first parameter, as for ast helpers that take a symbol.

// Go: ls/file_rename_test.go (tsgo#4900)
#[cfg(test)]
mod tests {
    use super::new_path_updater;

    // Go: ls/file_rename_test.go:45 TestCreatePathUpdaterCaseFoldingShrinksOldPath
    // TestCreatePathUpdaterCaseFoldingShrinksOldPath reproduces a panic that used to occur when
    // createPathUpdater confirmed a case-insensitive directory match via tspath.StartsWithDirectory,
    // then sliced the raw (non-canonicalized) file path using the raw byte length of oldPath. Each
    // Kelvin sign '\u212A' below case-folds to the single-byte 'k', so the raw oldPath is longer in
    // bytes (15) than path (12), even though path's canonical form is case-insensitively prefixed by
    // oldPath's canonical form. Slicing path[len(oldPath):] used to panic with "slice bounds out of
    // range [15:12]"; createPathUpdater must instead trim by rune count via
    // tspath.TrimFilePathPrefix.
    // PORT: Go builds `&LanguageService{host: caseInsensitiveHost{}}`, whose
    // only used method is `UseCaseSensitiveFileNames() == false`. A Rust
    // LanguageService needs a program, so the test calls the updater body
    // with that value.
    #[test]
    fn test_create_path_updater_case_folding_shrinks_old_path() {
        let old_path = "/a/\u{212A}\u{212A}\u{212A}\u{212A}";
        let new_path = "/a/new";
        let updater =
            new_path_updater(false /*useCaseSensitiveFileNames*/, old_path, new_path);

        let (updated, ok) = updater("/a/kkkk/x.ts");
        assert!(ok);
        assert_eq!(updated, "/a/new/x.ts");
    }
}
