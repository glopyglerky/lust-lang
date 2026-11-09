//! The parts of Go `internal/module`, `internal/core` (pattern.go),
//! `internal/stringutil` and `internal/outputpaths` that modulespecifiers
//! uses.
//!
//! PORT: the Go `module` and `outputpaths` packages are ported under
//! `src/frontend/`, which `lib.rs` does not compile, and they need the full
//! resolver. These are copies of the small functions this package calls.

use crate::prelude::*;

use super::semver;
use super::tspath;

// Go: module/util.go:17 IsApplicableVersionedTypesKey

// Go: module/util.go:69 UnmangleScopedPackageName
pub fn unmangle_scoped_package_name(package_name: &str) -> String {
    if let Some((before, after)) = package_name.split_once("__") {
        return format!("@{before}/{after}");
    }
    package_name.to_string()
}

// Go: module/util.go:81 GetPackageNameFromTypesPackageName
pub fn get_package_name_from_types_package_name(mangled_name: &str) -> String {
    if let Some(without_at_type_prefix) = mangled_name.strip_prefix("@types/") {
        return unmangle_scoped_package_name(without_at_type_prefix);
    }
    mangled_name.to_string()
}

// Go: module/util.go:58 MangleScopedPackageName

// Go: module/util.go:182 TryGetJSExtensionForFile
// TryGetJSExtensionForFile maps TS/JS/DTS extensions to the output JS-side extension.
// Returns an empty string if the extension is unsupported.

// Go: module/resolver.go:1916 GetConditions

// Go: core/pattern.go:5 Pattern
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Pattern {
    pub text: String,
    pub star_index: isize, // -1 for exact match
}

impl Pattern {
    // Go: core/pattern.go:18 IsValid
    pub fn is_valid(&self) -> bool {
        self.star_index == -1 || (self.star_index as usize) < self.text.len()
    }

    // Go: core/pattern.go:22 Matches
    // PORT: Go compares bytes. `text` and `candidate` are port forms, so
    // this compares their Go bytes (see `scanner_util::GO_STRING_MARKER`).
    // `star_index` is the port offset of the star.

}

// Go: core/pattern.go:10 TryParsePattern

// Go: core/pattern.go:41 FindBestPatternMatch

// Go: module/resolver.go:1977 ParsedPatterns
#[derive(Clone, Debug, Default)]
pub struct ParsedPatterns {
    matchable_string_set: FxHashSet<String>,
    patterns: Vec<Pattern>,
}

// Go: module/resolver.go:1986 TryParsePatterns

// Go: module/resolver.go:2025 MatchPatternOrExact

// Go: core/core.go:715 IndexAfter

// Go: core/core.go:867 CompareBooleans
// CompareBooleans treats true as greater than false.
pub fn compare_booleans(a: bool, b: bool) -> i32 {
    if a && !b {
        1
    } else if !a && b {
        -1
    } else {
        0
    }
}

// Go: stringutil/compare.go:74 HasPrefix
// PORT: compares the Go bytes of the port forms (see
// `scanner_util::GO_STRING_MARKER`).

// Go: stringutil/compare.go:84 HasSuffix
// PORT: see `has_prefix`.
// PORT: Go uses strings.EqualFold (Unicode simple folding). This compares
// ASCII case only, which matches for the ASCII paths in practice.

// Go: stringutil/compare.go:94 HasPrefixAndSuffixWithoutOverlap

// Go: outputpaths/outputpaths.go:9 OutputPathsHost
pub trait OutputPathsHost {
    fn common_source_directory(&self) -> String;
    /// #4712: the file extensions of the configured content mappers.
    fn content_mapper_extensions(&self) -> Vec<String>;
    fn get_current_directory(&self) -> String;
    fn use_case_sensitive_file_names(&self) -> bool;
}

// Go: outputpaths/outputpaths.go:101 GetOutputJSFileNameWorker

// Go: outputpaths/outputpaths.go:108 GetOutputDeclarationFileNameWorker

// Go: outputpaths/outputpaths.go:148 ChangeToDeclarationExtension (#4712)

// Go: outputpaths/outputpaths.go:116 GetOutputExtension
pub fn get_output_extension(file_name: &str, jsx: JsxEmit) -> &'static str {
    if tspath::file_extension_is(file_name, tspath::EXTENSION_JSON) {
        tspath::EXTENSION_JSON
    } else if jsx == JsxEmit::PRESERVE
        && tspath::file_extension_is_one_of(
            file_name,
            &[tspath::EXTENSION_JSX, tspath::EXTENSION_TSX],
        )
    {
        tspath::EXTENSION_JSX
    } else if tspath::file_extension_is_one_of(
        file_name,
        &[tspath::EXTENSION_MTS, tspath::EXTENSION_MJS],
    ) {
        tspath::EXTENSION_MJS
    } else if tspath::file_extension_is_one_of(
        file_name,
        &[tspath::EXTENSION_CTS, tspath::EXTENSION_CJS],
    ) {
        tspath::EXTENSION_CJS
    } else {
        tspath::EXTENSION_JS
    }
}

// Go: outputpaths/outputpaths.go:165 getOutputPathWithoutChangingExtension
