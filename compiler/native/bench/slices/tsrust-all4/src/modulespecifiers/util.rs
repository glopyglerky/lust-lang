//! Port of modulespecifiers/util.go.

use crate::prelude::*;

#[cfg(not(target_family = "wasm"))]
use std::sync::{Arc, LazyLock, PoisonError, RwLock};

#[cfg(not(target_family = "wasm"))]
use crate::gostd::regexp;

use super::deps;
use super::specifiers::{};
use super::tspath;
use super::types::*;

// Go: modulespecifiers/util.go:29 comparePathsByRedirect

// Go: modulespecifiers/util.go:42 PathIsBareSpecifier

// Go: modulespecifiers/util.go:19 regexPatternCacheKey
#[cfg(not(target_family = "wasm"))]
#[derive(Clone, PartialEq, Eq, Hash)]
struct RegexPatternCacheKey {
    pattern: String,
    case_insensitive: bool,
}

// Go: modulespecifiers/util.go:24 regexPatternCache
// PORT: Go guards the map with a sync.RWMutex. Checker threads can reach
// this code (the declaration emitter), so the port keeps the lock. A panic
// under the lock (an unported regexp feature) must not block later calls,
// so a poisoned lock is used as is.

// Go: modulespecifiers/util.go:46 IsExcludedByRegex

/// The wasm build runs tsc only, and tsc passes default user preferences
/// (the checker's node builder and `get_module_specifier`), so `excludes`
/// is always empty there. Only the language service fills it, from the
/// editor's `autoImportSpecifierExcludeRegexes`. This leaves the Go regexp
/// engine and the unicode tables that only it uses out of the wasm module.
// PORT: not in Go.
#[cfg(target_family = "wasm")]
pub fn is_excluded_by_regex(_module_specifier: &str, excludes: &[String]) -> bool {
    if !excludes.is_empty() {
        unreachable!("the wasm build has no regexp engine for autoImportSpecifierExcludeRegexes");
    }
    false
}

// Go: modulespecifiers/util.go:59 stringToRegex

// Go: modulespecifiers/util.go:136 ensurePathIsNonModuleName

// Go: modulespecifiers/util.go:143 GetJSExtensionForDeclarationFileExtension
pub fn get_js_extension_for_declaration_file_extension(ext: &str) -> String {
    match ext {
        tspath::EXTENSION_DTS => tspath::EXTENSION_JS.to_string(),
        tspath::EXTENSION_DMTS => tspath::EXTENSION_MJS.to_string(),
        tspath::EXTENSION_DCTS => tspath::EXTENSION_CJS.to_string(),
        // .d.json.ts and the like
        _ => ext[".d".len()..ext.len() - tspath::EXTENSION_TS.len()].to_string(),
    }
}

// Go: modulespecifiers/util.go:159 TryGetRealFileNameForNonJSDeclarationFileName

// Go: modulespecifiers/util.go:174 getJSExtensionForFile

// Go: modulespecifiers/util.go:186 extensionFromPath
/// Gets the extension from a path. Path must have a valid extension.
fn extension_from_path(path: &str) -> &'static str {
    let ext = tspath::try_get_extension_from_path(path);
    if ext.is_empty() {
        panic!("File {path} has unknown extension.");
    }
    ext
}

// Go: modulespecifiers/util.go:194 tryGetAnyFileFromPath

// Go: modulespecifiers/util.go:214 getPathsRelativeToRootDirs

// Go: modulespecifiers/util.go:225 isPathRelativeToParent
pub(crate) fn is_path_relative_to_parent(path: &str) -> bool {
    path.starts_with("..")
}

// Go: modulespecifiers/util.go:229 getRelativePathIfInSameVolume

// Go: modulespecifiers/util.go:240 packageJsonPathsAreEqual

// Go: modulespecifiers/util.go:250 prefersTsExtension
pub(crate) fn prefers_ts_extension(allowed_endings: &[ModuleSpecifierEnding]) -> bool {
    let js_priority = index_of(allowed_endings, ModuleSpecifierEnding::JsExtension);
    let ts_priority = index_of(allowed_endings, ModuleSpecifierEnding::TsExtension);
    if ts_priority > -1 {
        return ts_priority < js_priority;
    }
    false
}

/// Go `slices.Index`: the first index of `value`, or -1.
pub(crate) fn index_of(endings: &[ModuleSpecifierEnding], value: ModuleSpecifierEnding) -> isize {
    endings
        .iter()
        .position(|e| *e == value)
        .map_or(-1, |i| i as isize)
}

// Go: modulespecifiers/util.go:259 replaceFirstStar
// PORT: Go joins the bytes (see `scanner_util::go_value`).

// Go: modulespecifiers/util.go:263 NodeModulePathParts
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NodeModulePathParts {
    pub top_level_node_modules_index: isize,
    pub top_level_package_name_index: isize,
    pub package_root_index: isize,
    pub file_name_index: isize,
}

// Go: modulespecifiers/util.go:270 nodeModulesPathParseState
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum NodeModulesPathParseState {
    BeforeNodeModules,
    NodeModules,
    Scope,
    PackageContent,
}

// Go: modulespecifiers/util.go:279 GetNodeModulePathParts

// Go: modulespecifiers/util.go:332 GetNodeModulesPackageName

// Go: modulespecifiers/util.go:350 allKeysStartWithDot

// Go: modulespecifiers/util.go:359 GetPackageNameFromDirectory

// Go: modulespecifiers/util.go:388 ProcessEntrypointEnding
// PORT: not ported. It takes a `module.ResolvedEntrypoint`, which only the
// language service auto-import code produces.
