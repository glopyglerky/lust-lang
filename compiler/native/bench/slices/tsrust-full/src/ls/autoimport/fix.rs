use crate::ls::autoimport::prelude::*;

// Port of Go `ls/autoimport/fix.go`.
//
// PORT (whole file):
// - Go `*Fix` values are shared (`Rc<Fix>`). Go embeds
//   `*lsproto.AutoImportFix`; it is the field `auto_import_fix`, and `Deref`
//   promotes its fields (`fix.kind`, `fix.name`) as Go does.
// - Go `*newImportBinding` values are never changed after they are made, so
//   they are `NewImportBinding` values (`Option` where Go can pass nil).
// - `(*View).GetFixes` and the methods it calls take the request checker
//   `ch`. Go stores it in the view (`v.checker`, `NewView`'s `typeChecker`,
//   ts#64178); the Rust checker is a `RefCell` that the caller already
//   borrows, so the caller passes it to each method that reads it (as the
//   pinned ImportAdder decision does for the adder).
// - Go passes `*ast.SourceFile` to program methods that take an
//   `ast.HasFileName`: `source_file_has_file_name(file)` (view.rs).
// - The change tracker's embedded Go `NodeFactory` is `ct.node_factory()`.
//   Nodes are made into locals first, in Go argument order, before the
//   tracker call that takes `&mut ct`.
// - Go `panic` text that prints a `Kind` (`KindString()`, `%v`) prints
//   `debug::kind_string` ("KindX").

use crate::frontend::compiler;
use crate::frontend::core_ls_ext::compare_booleans;
use crate::frontend::scanner::scanner_ls;
use crate::frontend::tspath;
use crate::locale;
use crate::ls::{change, lsconv, lsutil};
use crate::lsp::lsproto;
use crate::modulespecifiers;

use crate::flags_macros::go_enum;

// Go: ls/autoimport/fix.go:30 newImportBinding
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NewImportBinding {
    pub kind: lsproto::ImportKind,
    pub property_name: String,
    pub name: String,
    pub add_as_type_only: lsproto::AddAsTypeOnly,
}

// Go: ls/autoimport/fix.go:37 Fix
#[derive(Clone, Debug, Default)]
pub struct Fix {
    pub auto_import_fix: lsproto::AutoImportFix,

    pub module_specifier_kind: modulespecifiers::ResultKind,
    pub is_re_export: bool,
    pub module_file_name: String,
    pub type_only_alias_declaration: Node,
}

impl std::ops::Deref for Fix {
    type Target = lsproto::AutoImportFix;
    fn deref(&self) -> &lsproto::AutoImportFix {
        &self.auto_import_fix
    }
}

impl std::ops::DerefMut for Fix {
    fn deref_mut(&mut self) -> &mut lsproto::AutoImportFix {
        &mut self.auto_import_fix
    }
}

// Go: ls/autoimport/fix.go:46 addToExistingImportFix
#[derive(Clone, Debug, Default)]
pub struct AddToExistingImportFix {
    pub import_clause_or_binding_pattern: Node,
    // One of `defaultImport` or `namedImports` will be present
    pub default_import: Option<NewImportBinding>,
    pub named_import: Option<NewImportBinding>,
}

impl Fix {
    // Go: ls/autoimport/fix.go:56 Edits
    // Edits produces the text edits and a human-readable description for the fix. The returned bool is false
    // when the fix targets a content-mapped file and any edit could not be placed within a single verbatim
    // span, meaning it cannot be safely applied to the original text and the caller should discard it.
}

// Go: ls/autoimport/fix.go:133 fileEdits
// fileEdits returns the edits recorded for file, along with whether they are safe to apply. GetChanges
// drops the edits of any content-mapped file that cannot be faithfully mapped back to the original text,
// so an empty result with safe == false means the fix could not be represented and must be discarded.
// PORT: Go `changes[name]` is a nil slice for a missing key; that is an empty `Vec`.
// Go `file.OriginalFileName()` is the `lsconv::Script` method of a file root `Node`.

// Go: ls/autoimport/fix.go:138 addImportType

// Go: ls/autoimport/fix.go:152 addNamespaceQualifier

// Go: ls/autoimport/fix.go:161 getAddToExistingImportFix

// Go: ls/autoimport/fix.go:198 addToExistingImport

// Go: ls/autoimport/fix.go:321 getTypeKeywordOfTypeOnlyImport

// Go: ls/autoimport/fix.go:331 addElementToBindingPattern
// PORT: Go `core.IfElse` evaluates both arguments, so the property name
// identifier is made even when `propertyName` is empty (and then unused).
// Go passes that identifier as the binding element's initializer.

// Go: ls/autoimport/fix.go:346 getNewImports

// Go: ls/autoimport/fix.go:415 getNewRequires
// PORT: Go does not read `compilerOptions`.

// Go: ls/autoimport/fix.go:480 createConstEqualsRequireDeclaration

// Go: ls/autoimport/fix.go:503 insertImports
// PORT: Go `comparer` is a func value that is never nil here
// (`detectCaseSensitivityBySort` falls back to the first comparer); a nil
// value panics when called, as in Go.

// Go: ls/autoimport/fix.go:542 makeImport

/// Go `unicode.IsUpper`: general category Lu.
// PORT: the Rust `Uppercase` property is Lu plus `Other_Uppercase`; the
// `Other_Uppercase` ranges are removed so the result is Go's category test
// (the same helper as in `util.rs`, which keeps it private).
fn unicode_is_upper(c: char) -> bool {
    if !c.is_uppercase() {
        return false;
    }
    !matches!(
        c as u32,
        0x2160..=0x216F | 0x24B6..=0x24CF | 0x1F130..=0x1F149 | 0x1F150..=0x1F169 | 0x1F170..=0x1F189
    )
}



// Go: ls/autoimport/fix.go:623 getAddAsTypeOnly
// getAddAsTypeOnly determines if an import should be type-only based on usage context


// Go: ls/autoimport/fix.go:669 getNamespaceLikeImportText


// Go: ls/autoimport/fix.go:796 GetImportKindForImportStatement
// The completions of an import statement (`import F|`) call this. It always
// writes an `import` keyword, so an `export =` module in a JS file without an
// external module indicator gives a default import, not `require`.

// Go: ls/autoimport/fix.go:800 getImportKind
// PORT: Go `fallthrough` from the Named case into the Modifier case is the
// shared `NAMED` result.

// Go: ls/autoimport/fix.go:840 existingImport
#[derive(Clone, Debug, Default)]
pub struct ExistingImport {
    pub node: Node,
    pub module_specifier: String,
    pub index: i32,
}


// Go: ls/autoimport/fix.go:885 fileSyntaxKind
// fileSyntaxKind represents the detected module syntax of a source file.
go_enum!(FileSyntaxKind, i32 {
    AMBIGUOUS = 0; // fileSyntaxKindAmbiguous
    ESM = 1; // fileSyntaxKindESM
    CJS = 2; // fileSyntaxKindCJS
});

// Go: ls/autoimport/fix.go:897 detectSyntax
// detectSyntax returns whether a source file has unambiguous ESM or CJS syntax.
// When moduleDetection is "force", ExternalModuleIndicator may be set to the
// source file node itself rather than a genuine syntax indicator, so we fall back
// to inspecting the file's Imports() to find actual import/export declarations.

// Go: ls/autoimport/fix.go:913 detectSyntaxIndicators
// detectSyntaxIndicators checks whether a source file contains genuine ESM
// and/or CJS syntax. Under moduleDetection "force", the cached
// ExternalModuleIndicator may be the source file itself rather than a real
// statement, so we look at Imports() for actual import/export declarations.


// Go: ls/autoimport/fix.go:993 needsTypeOnly
fn needs_type_only(add_as_type_only: lsproto::AddAsTypeOnly) -> bool {
    add_as_type_only == lsproto::AddAsTypeOnly::REQUIRED
}

// Go: ls/autoimport/fix.go:997 shouldUseTypeOnly


// Go: ls/autoimport/fix.go:1022 compareFixKinds
fn compare_fix_kinds(a: lsproto::AutoImportFixKind, b: lsproto::AutoImportFixKind) -> i32 {
    a.0 - b.0
}


// Go: ls/autoimport/fix.go:1094 isFixPossiblyReExportingImportingFile
// This is a simple heuristic to try to avoid creating an import cycle with a barrel re-export.
// E.g., do not `import { Foo } from ".."` when you could `import { Foo } from "../Foo"`.
// This can produce false positives or negatives if re-exports cross into sibling directories
// (e.g. `export * from "../whatever"`) or are not named "index". Technically this should do
// a tspath.Path comparison, but it's not worth it to run a heuristic in such a hot path.

// Go: ls/autoimport/fix.go:1102 isIndexFileName

// Go: ls/autoimport/fix.go:1115 promoteFromTypeOnly

// Go: ls/autoimport/fix.go:1208 promoteImportClause
// promoteImportClause removes the type keyword from an import clause

// Go: ls/autoimport/fix.go:1289 deleteTypeKeyword
// deleteTypeKeyword deletes the 'type' keyword token starting at the given position,
// including any trailing whitespace.

// Go: ls/autoimport/fix.go:1304 getModuleSpecifierText

// Go: ls/autoimport/fix.go:1326 compareModuleSpecifierRelativity
// returns `-1` if `a` is better than `b`
fn compare_module_specifier_relativity(
    a: &Fix,
    b: &Fix,
    preferences: &modulespecifiers::UserPreferences,
) -> i32 {
    match preferences.import_module_specifier_preference {
        modulespecifiers::ImportModuleSpecifierPreference::NonRelative
        | modulespecifiers::ImportModuleSpecifierPreference::ProjectRelative => compare_booleans(
            a.module_specifier_kind == modulespecifiers::ResultKind::Relative,
            b.module_specifier_kind == modulespecifiers::ResultKind::Relative,
        ),
        _ => 0,
    }
}
