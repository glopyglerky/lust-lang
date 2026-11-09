//! Port of module/util.go.

use crate::frontend::prelude::*;
use std::sync::LazyLock;

// Go: module/util.go:13 typeScriptVersion
// PORT: Go package-level var is a `LazyLock`. It stays private, because
// packagejson has its own copy with the same Go name.

// Go: module/util.go:15 InferredTypesContainingFile
pub const INFERRED_TYPES_CONTAINING_FILE: &str = "__inferred type names__.ts";

// Go: module/util.go:17 IsApplicableVersionedTypesKey

// Go: module/util.go:28 ParseNodeModuleFromPath

// Go: module/util.go:43 ParsePackageName

// Go: module/util.go:58 MangleScopedPackageName

// Go: module/util.go:69 UnmangleScopedPackageName
#[must_use]
pub fn unmangle_scoped_package_name(package_name: &str) -> String {
    if let Some((before, after)) = package_name.split_once("__") {
        return format!("@{before}/{after}");
    }
    package_name.to_string()
}

// Go: module/util.go:77 GetTypesPackageName

// Go: module/util.go:81 GetPackageNameFromTypesPackageName
#[must_use]
pub fn get_package_name_from_types_package_name(mangled_name: &str) -> String {
    if let Some(without_at_type_prefix) = mangled_name.strip_prefix("@types/") {
        return unmangle_scoped_package_name(without_at_type_prefix);
    }
    mangled_name.to_string()
}

// Go: module/util.go:89 ComparePatternKeys

// Go: module/util.go:125 GetResolutionDiagnostic
// Returns a DiagnosticMessage if we won't include a resolved module due to its extension.
// The DiagnosticMessage's parameters are the imported module name, and the filename it resolved to.
// This returns a diagnostic even if the module will be an untyped module.
// PORT: Go `*ast.SourceFile` is the loader's `ParsedSourceFile`. The only
// caller runs during program construction, before `source_file_info` exists.

// Go: module/util.go:178 TryGetJSExtensionForFile
// TryGetJSExtensionForFile maps TS/JS/DTS extensions to the output JS-side extension.
// Returns an empty string if the extension is unsupported.
