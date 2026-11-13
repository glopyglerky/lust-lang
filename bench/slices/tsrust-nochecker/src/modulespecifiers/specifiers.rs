//! Port of modulespecifiers/specifiers.go.

use crate::prelude::*;

use super::deps;
use super::packagejson::{};
use super::preferences::{ModuleSpecifierPreferences, };
use super::tspath;
use super::types::*;
use super::util::*;

// Go: modulespecifiers/specifiers.go:19 GetModuleSpecifiers
// PORT: `checker` is `&mut` because the Rust checker methods take `&mut self`.

// Go: modulespecifiers/specifiers.go:41 GetModuleSpecifiersWithInfo

// Go: modulespecifiers/specifiers.go:79 GetModuleSpecifiersForFileWithInfo

// Go: modulespecifiers/specifiers.go:107 ambientModuleInfo (ts#63931)

// Go: modulespecifiers/specifiers.go:112 tryGetModuleNameFromAmbientModule

// Go: modulespecifiers/specifiers.go:162 Info
#[derive(Clone, Debug, Default)]
pub struct Info {
    pub use_case_sensitive_file_names: bool,
    pub importing_source_file_name: String,
    pub source_directory: String,
}

// Go: modulespecifiers/specifiers.go:168 getInfo

// Go: modulespecifiers/specifiers.go:180 getAllModulePaths

// Go: modulespecifiers/specifiers.go:203 getAllModulePathsWorker
// PORT: Go collects paths in a map with random iteration order and sorts
// each group. The IndexMap keeps insertion order; the sort result is the
// same because `compare_paths_by_redirect` ends with a path comparison.

// Go: modulespecifiers/specifiers.go:252 containsIgnoredPath
// containsIgnoredPath checks if a path contains patterns that should be ignored.
// This is a local helper that duplicates tspath.ContainsIgnoredPath for performance.
fn contains_ignored_path(s: &str) -> bool {
    s.contains("/node_modules/.") || s.contains("/.git") || s.contains(".#")
}

// Go: modulespecifiers/specifiers.go:259 ContainsNodeModules
/// Checks if a path contains the node_modules directory.
pub fn contains_node_modules(s: &str) -> bool {
    s.contains("/node_modules/")
}

// Go: modulespecifiers/specifiers.go:265 GetEachFileNameOfModule

// Go: modulespecifiers/specifiers.go:364 computeModuleSpecifiers

// Go: modulespecifiers/specifiers.go:490 getLocalModuleSpecifier

// Go: modulespecifiers/specifiers.go:641 processEnding
// PORT: Go checks `host != nil` before probing the file system. Every Go
// caller passes a host, so `host` is not optional here.

// Go: modulespecifiers/specifiers.go:712 tryGetModuleNameFromRootDirs

// Go: modulespecifiers/specifiers.go:748 tryGetModuleNameAsNodeModule

// Go: modulespecifiers/specifiers.go:830 pkgJsonDirAttemptResult
#[derive(Clone, Debug, Default)]
struct PkgJsonDirAttemptResult {
    module_file_to_try: String,
    package_root_path: String,
    blocked_by_exports: bool,
    verbatim_from_exports: bool,
}

// Go: modulespecifiers/specifiers.go:837 tryDirectoryWithPackageJson

// Go: modulespecifiers/specifiers.go:981 tryGetModuleNameFromExports

// Go: modulespecifiers/specifiers.go:1024 tryGetModuleNameFromPackageJsonImports

// Go: modulespecifiers/specifiers.go:1089 specPair
#[derive(Clone, Debug)]
struct SpecPair {
    ending: ModuleSpecifierEnding,
    value: String,
}

// Go: modulespecifiers/specifiers.go:1094 tryGetModuleNameFromPaths

// Go: modulespecifiers/specifiers.go:1193 validateEnding

// Go: modulespecifiers/specifiers.go:1204 tryGetModuleNameFromExportsOrImports

// Go: modulespecifiers/specifiers.go:1333 GetModuleSpecifier
// `importingSourceFile` and `importingSourceFileName`? Why not just use `importingSourceFile.path`?
// Because when this is called by the declaration emitter, `importingSourceFile` is the implementation
// file, but `importingSourceFileName` and `toFileName` refer to declaration files (the former to the
// one currently being produced; the latter to the one being imported). We need an implementation file
// just to get its `impliedNodeFormat` and to detect certain preferences from existing import module
// specifiers.

// Go: modulespecifiers/specifiers.go:1354 UpdateModuleSpecifier

// Go: modulespecifiers/specifiers.go:1376 getModuleSpecifierWithPreferences
