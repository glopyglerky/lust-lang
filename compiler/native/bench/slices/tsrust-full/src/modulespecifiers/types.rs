//! Port of modulespecifiers/types.go.

use crate::prelude::*;

use super::tspath;
use std::sync::Arc;

// Go: modulespecifiers/types.go:13 SourceFileForSpecifierGeneration
pub trait SourceFileForSpecifierGeneration {
    fn path(&self) -> tspath::Path;
    fn file_name(&self) -> String;
    fn imports(&self) -> Vec<Node>;
    fn is_js(&self) -> bool;
    // PORT: Go passes the same value to host methods that take an
    // `ast.HasFileName`. Host methods here take the source file node.
    fn node(&self) -> Node;
}


// Go: modulespecifiers/types.go:20 CheckerShape
// PORT: the checker methods take `&mut self`. Go reads
// `moduleSymbol.Declarations` and other symbol fields directly; `symbols`
// gives that access.


// Go: modulespecifiers/types.go:25 ResultKind
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ResultKind {
    #[default]
    None,
    NodeModules,
    Paths,
    Redirect,
    Relative,
    Ambient,
}

// Go: modulespecifiers/types.go:36 ModuleSpecifiersResult (ts#63931)

// Go: modulespecifiers/types.go:42 ModulePath
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModulePath {
    pub file_name: String,
    pub is_in_node_modules: bool,
    pub is_redirect: bool,
}

// Go: modulespecifiers/types.go:48 ModuleSpecifierGenerationHost
// PORT: methods that take `ast.HasFileName` or `*ast.StringLiteralLike`
// take a `Node`. Package.json entries are shared through `Arc`, as Go
// shares the `*packagejson.InfoCacheEntry` pointer.

// Go: modulespecifiers/types.go:71 ImportModuleSpecifierPreference
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImportModuleSpecifierPreference {
    #[default]
    None, // "" !!!
    Shortest,        // "shortest"
    ProjectRelative, // "project-relative"
    Relative,        // "relative"
    NonRelative,     // "non-relative"
}

// Go: modulespecifiers/types.go:81 ImportModuleSpecifierEndingPreference
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImportModuleSpecifierEndingPreference {
    #[default]
    None, // "" !!!
    Auto,    // "auto"
    Minimal, // "minimal"
    Index,   // "index"
    Js,      // "js"
}

// Go: modulespecifiers/types.go:91 UserPreferences
#[derive(Clone, Debug, Default)]
pub struct UserPreferences {
    pub import_module_specifier_preference: ImportModuleSpecifierPreference,
    pub import_module_specifier_ending: ImportModuleSpecifierEndingPreference,
    pub auto_import_specifier_exclude_regexes: Vec<String>,
}

// Go: modulespecifiers/types.go:97 ModuleSpecifierOptions
#[derive(Clone, Copy, Debug, Default)]
pub struct ModuleSpecifierOptions {
    pub override_import_mode: ResolutionMode,
}

// Go: modulespecifiers/types.go:101 RelativePreferenceKind
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelativePreferenceKind {
    Relative,
    NonRelative,
    Shortest,
    ExternalNonRelative,
}

// Go: modulespecifiers/types.go:110 ModuleSpecifierEnding
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModuleSpecifierEnding {
    Minimal,
    Index,
    JsExtension,
    TsExtension,
}

// Go: modulespecifiers/types.go:119 MatchingMode
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchingMode {
    Exact,
    Directory,
    Pattern,
}
