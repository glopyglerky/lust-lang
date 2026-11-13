//! Port of tspath/extension.go.
//!
//! PORT: a copy of `src/frontend/tspath/extension.rs`. That file finds the
//! path helpers through the `frontend` prelude, which `lib.rs` does not
//! compile. This copy imports them from `tspath_path`. Keep the two in sync.

use crate::prelude::*;

use super::tspath_path::*;

// Go: tspath/extension.go:8 Extension constants
pub const EXTENSION_TS: &str = ".ts";
pub const EXTENSION_TSX: &str = ".tsx";
pub const EXTENSION_DTS: &str = ".d.ts";
pub const EXTENSION_JS: &str = ".js";
pub const EXTENSION_JSX: &str = ".jsx";
pub const EXTENSION_JSON: &str = ".json";
pub const EXTENSION_TS_BUILD_INFO: &str = ".tsbuildinfo";
pub const EXTENSION_MJS: &str = ".mjs";
pub const EXTENSION_MTS: &str = ".mts";
pub const EXTENSION_DMTS: &str = ".d.mts";
pub const EXTENSION_CJS: &str = ".cjs";
pub const EXTENSION_CTS: &str = ".cts";
pub const EXTENSION_DCTS: &str = ".d.cts";

// Go: tspath/extension.go:24 extension lists
// PORT: Go `slices.Concat` values are written out as constant lists with the
// same order.
pub const SUPPORTED_DECLARATION_EXTENSIONS: &[&str] =
    &[EXTENSION_DTS, EXTENSION_DCTS, EXTENSION_DMTS];
pub const SUPPORTED_TS_IMPLEMENTATION_EXTENSIONS: &[&str] =
    &[EXTENSION_TS, EXTENSION_TSX, EXTENSION_MTS, EXTENSION_CTS];
const SUPPORTED_TS_EXTENSIONS_FOR_EXTRACT_EXTENSION: &[&str] = &[
    EXTENSION_DTS,
    EXTENSION_DCTS,
    EXTENSION_DMTS,
    EXTENSION_TS,
    EXTENSION_TSX,
    EXTENSION_MTS,
    EXTENSION_CTS,
];
pub const ALL_SUPPORTED_EXTENSIONS: &[&[&str]] = &[
    &[
        EXTENSION_TS,
        EXTENSION_TSX,
        EXTENSION_DTS,
        EXTENSION_JS,
        EXTENSION_JSX,
    ],
    &[EXTENSION_CTS, EXTENSION_DCTS, EXTENSION_CJS],
    &[EXTENSION_MTS, EXTENSION_DMTS, EXTENSION_MJS],
];
pub const SUPPORTED_TS_EXTENSIONS: &[&[&str]] = &[
    &[EXTENSION_TS, EXTENSION_TSX, EXTENSION_DTS],
    &[EXTENSION_CTS, EXTENSION_DCTS],
    &[EXTENSION_MTS, EXTENSION_DMTS],
];
pub const SUPPORTED_TS_EXTENSIONS_FLAT: &[&str] = &[
    EXTENSION_TS,
    EXTENSION_TSX,
    EXTENSION_DTS,
    EXTENSION_CTS,
    EXTENSION_DCTS,
    EXTENSION_MTS,
    EXTENSION_DMTS,
];
pub const SUPPORTED_JS_EXTENSIONS: &[&[&str]] = &[
    &[EXTENSION_JS, EXTENSION_JSX],
    &[EXTENSION_MJS],
    &[EXTENSION_CJS],
];
pub const SUPPORTED_JS_EXTENSIONS_FLAT: &[&str] =
    &[EXTENSION_JS, EXTENSION_JSX, EXTENSION_MJS, EXTENSION_CJS];
pub const ALL_SUPPORTED_EXTENSIONS_WITH_JSON: &[&[&str]] = &[
    &[
        EXTENSION_TS,
        EXTENSION_TSX,
        EXTENSION_DTS,
        EXTENSION_JS,
        EXTENSION_JSX,
    ],
    &[EXTENSION_CTS, EXTENSION_DCTS, EXTENSION_CJS],
    &[EXTENSION_MTS, EXTENSION_DMTS, EXTENSION_MJS],
    &[EXTENSION_JSON],
];
pub const SUPPORTED_TS_EXTENSIONS_WITH_JSON: &[&[&str]] = &[
    &[EXTENSION_TS, EXTENSION_TSX, EXTENSION_DTS],
    &[EXTENSION_CTS, EXTENSION_DCTS],
    &[EXTENSION_MTS, EXTENSION_DMTS],
    &[EXTENSION_JSON],
];
pub const SUPPORTED_TS_EXTENSIONS_WITH_JSON_FLAT: &[&str] = &[
    EXTENSION_TS,
    EXTENSION_TSX,
    EXTENSION_DTS,
    EXTENSION_CTS,
    EXTENSION_DCTS,
    EXTENSION_MTS,
    EXTENSION_DMTS,
    EXTENSION_JSON,
];
pub const EXTENSIONS_NOT_SUPPORTING_EXTENSIONLESS_RESOLUTION: &[&str] = &[
    EXTENSION_MTS,
    EXTENSION_DMTS,
    EXTENSION_MJS,
    EXTENSION_CTS,
    EXTENSION_DCTS,
    EXTENSION_CJS,
];

// Go: tspath/extension.go:39 ExtensionIsTs

// Go: tspath/extension.go:43 extensionsToRemove
const EXTENSIONS_TO_REMOVE: &[&str] = &[
    EXTENSION_DTS,
    EXTENSION_DMTS,
    EXTENSION_DCTS,
    EXTENSION_MJS,
    EXTENSION_MTS,
    EXTENSION_CJS,
    EXTENSION_CTS,
    EXTENSION_TS,
    EXTENSION_JS,
    EXTENSION_TSX,
    EXTENSION_JSX,
    EXTENSION_JSON,
];

// Go: tspath/extension.go:45 RemoveFileExtension
pub fn remove_file_extension(path: &str) -> &str {
    // Remove any known extension even if it has more than one dot
    for ext in EXTENSIONS_TO_REMOVE {
        if let Some(stripped) = path.strip_suffix(ext) {
            return stripped;
        }
    }

    path
}

// Go: tspath/extension.go:66 TryGetExtensionFromPath
pub fn try_get_extension_from_path(p: &str) -> &'static str {
    for ext in EXTENSIONS_TO_REMOVE {
        if file_extension_is(p, ext) {
            return ext;
        }
    }
    ""
}

// Go: tspath/extension.go:75 RemoveExtension
pub fn remove_extension<'a>(path: &'a str, extension: &str) -> &'a str {
    &path[..path.len() - extension.len()]
}

// Go: tspath/extension.go:79 FileExtensionIsOneOf
pub fn file_extension_is_one_of(path: &str, extensions: &[&str]) -> bool {
    for ext in extensions {
        if file_extension_is(path, ext) {
            return true;
        }
    }
    false
}

// Go: tspath/extension.go:88 TryExtractTSExtension
pub fn try_extract_ts_extension(file_name: &str) -> &'static str {
    for ext in SUPPORTED_TS_EXTENSIONS_FOR_EXTRACT_EXTENSION {
        if file_extension_is(file_name, ext) {
            return ext;
        }
    }
    ""
}

// Go: tspath/extension.go:97 HasTSFileExtension
pub fn has_ts_file_extension(path: &str) -> bool {
    file_extension_is_one_of(path, SUPPORTED_TS_EXTENSIONS_FLAT)
}

// Go: tspath/extension.go:101 HasImplementationTSFileExtension

// Go: tspath/extension.go:105 HasJSFileExtension
pub fn has_js_file_extension(path: &str) -> bool {
    file_extension_is_one_of(path, SUPPORTED_JS_EXTENSIONS_FLAT)
}

// Go: tspath/extension.go:109 HasJSONFileExtension
pub fn has_json_file_extension(path: &str) -> bool {
    file_extension_is(path, EXTENSION_JSON)
}

// Go: tspath/extension.go:113 IsDeclarationFileName

// Go: tspath/extension.go:117 ExtensionIsOneOf
pub fn extension_is_one_of(ext: &str, extensions: &[&str]) -> bool {
    extensions.contains(&ext)
}

// Go: tspath/extension.go:121 GetDeclarationFileExtension

// Go: tspath/extension.go:137 GetDeclarationEmitExtensionForPath

// Go: tspath/extension.go:159 ChangeAnyExtension
// ChangeAnyExtension changes the extension of a path to the provided extension if it has one of the provided extensions.
//
// ChangeAnyExtension("/path/to/file.ext", ".js", ".ext") === "/path/to/file.js"
// ChangeAnyExtension("/path/to/file.ext", ".js", ".ts") === "/path/to/file.ext"
// ChangeAnyExtension("/path/to/file.ext", ".js", [".ext", ".ts"]) === "/path/to/file.js"

// Go: tspath/extension.go:174 ChangeExtension

// Go: tspath/extension.go:183 ChangeFullExtension
// Like `changeAnyExtension`, but declaration file extensions are recognized
// and replaced starting from the `.d`.
//
//	changeAnyExtension("file.d.ts", ".js") === "file.d.js"
//	changeFullExtension("file.d.ts", ".js") === "file.js"

// Go: tspath/extension.go:195 GetPossibleOriginalInputExtensionForExtension
