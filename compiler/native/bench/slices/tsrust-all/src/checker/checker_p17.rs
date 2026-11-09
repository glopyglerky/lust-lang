//! Port of Go `checker/checker.go` lines 14829-15734 (alias targets,
//! external module resolution, ES module symbol wrapping, entity names).
//!
//! PORT: Go `c.error(...)` returns the `*ast.Diagnostic` it already added, and
//! some callers mutate it afterwards (`AddRelatedInfo`). Diagnostics are owned
//! values here, so those call sites build the diagnostic with
//! `new_diagnostic_for_node`, finish it, then call `self.add_diagnostic` (the
//! same two steps Go `error` does).
//!
//! PORT: Go `tspath`, `module.GetResolutionDiagnostic` and a few `core`
//! helpers have no port in this crate. Private copies that follow the Go code
//! exactly live in the `tspath_p17`, `module_p17` and `core_p17` modules at
//! the end of this file.
//!
//! PORT: Go `c.program.X(...)` methods are free functions with the Go snake
//! names (see `PORTING.md`, Program section). `*module.ResolvedModule` is
//! `Option<&ResolvedModule>`.

use crate::prelude::*;

// PORT: Go compares `*diagnostics.Message` pointers. A nil message is `None`.

// PORT: Go `resolvedModule.IsResolved()` (nil-safe method on `*ResolvedModule`).


// Go: checker/checker.go:15294 getModuleSpecifierFromNode


// Go: checker/checker.go:15744 resolutionExtensionIsTSOrJson
pub fn resolution_extension_is_ts_or_json(ext: &str) -> bool {
    tspath_p17::extension_is_ts(ext) || ext == tspath_p17::EXTENSION_JSON
}


// Go: checker/checker.go:15947 isESMFormatImportImportingCommonjsFormatFile
pub fn is_esm_format_import_importing_commonjs_format_file(
    usage_mode: ResolutionMode,
    target_mode: ResolutionMode,
) -> bool {
    usage_mode == ModuleKind::ES_NEXT && target_mode == ModuleKind::COMMON_JS
}


// PORT: Go `maps.Clone(table)` on an `ast.SymbolTable`. A nil map clones to
// nil; otherwise a new table with the same entries.

/// Private copies of Go `tspath` functions used by this unit. tspath has no
/// Go-shaped port in this crate, so these follow the Go code exactly.
pub(crate) mod tspath_p17 {
    // Go: tspath/extension.go constants
    pub const EXTENSION_TS: &str = ".ts";
    pub const EXTENSION_TSX: &str = ".tsx";
    pub const EXTENSION_DTS: &str = ".d.ts";
    pub const EXTENSION_JS: &str = ".js";
    pub const EXTENSION_JSX: &str = ".jsx";
    pub const EXTENSION_JSON: &str = ".json";
    pub const EXTENSION_MJS: &str = ".mjs";
    pub const EXTENSION_MTS: &str = ".mts";
    pub const EXTENSION_DMTS: &str = ".d.mts";
    pub const EXTENSION_CJS: &str = ".cjs";
    pub const EXTENSION_CTS: &str = ".cts";
    pub const EXTENSION_DCTS: &str = ".d.cts";

    const SUPPORTED_DECLARATION_EXTENSIONS: [&str; 3] =
        [EXTENSION_DTS, EXTENSION_DCTS, EXTENSION_DMTS];
    const SUPPORTED_TS_EXTENSIONS_FOR_EXTRACT_EXTENSION: [&str; 7] = [
        EXTENSION_DTS,
        EXTENSION_DCTS,
        EXTENSION_DMTS,
        EXTENSION_TS,
        EXTENSION_TSX,
        EXTENSION_MTS,
        EXTENSION_CTS,
    ];
    pub const SUPPORTED_TS_EXTENSIONS_FLAT: [&str; 7] = [
        EXTENSION_TS,
        EXTENSION_TSX,
        EXTENSION_DTS,
        EXTENSION_CTS,
        EXTENSION_DCTS,
        EXTENSION_MTS,
        EXTENSION_DMTS,
    ];
    const EXTENSIONS_TO_REMOVE: [&str; 12] = [
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

    // Go: tspath/path.go:988 ComparePathsOptions
    pub struct ComparePathsOptions {
        pub use_case_sensitive_file_names: bool,
        pub current_directory: String,
    }

    // Go: tspath/path.go:997 ComparePathsOptions.getEqualityComparer
    // PORT: `stringutil.GetStringEqualityComparer(!o.UseCaseSensitiveFileNames)`.

    // Go: stringutil/compare.go:9 EquateStringCaseInsensitive
    // PORT: Go `strings.EqualFold` uses Unicode simple case folding. This
    // compares each rune by its simple lowercase and uppercase forms, which
    // gives the same result for every rune with a one-to-one case mapping.

    // Go: tspath/path.go:27 isAnyDirectorySeparator
    fn is_any_directory_separator(ch: u8) -> bool {
        ch == b'/' || ch == b'\\'
    }

    // Go: tspath/path.go:38 IsRootedDiskPath

    // Go: tspath/path.go:67 PathIsAbsolute

    // Go: tspath/path.go:71 HasTrailingDirectorySeparator
    fn has_trailing_directory_separator(path: &str) -> bool {
        !path.is_empty() && is_any_directory_separator(path.as_bytes()[path.len() - 1])
    }

    // Go: tspath/path.go:91 CombinePaths
    // PORT: Go builds into one `strings.Builder` and slices from `start`; the
    // result string is the same as building the current result directly.

    // Go: tspath/path.go:134 GetPathComponents

    // Go: tspath/path.go:139 pathComponents

    // Go: tspath/path.go:148 IsVolumeCharacter

    // Go: tspath/path.go:152 getFileUrlVolumeSeparatorEnd
    fn get_file_url_volume_separator_end(url: &[u8], start: usize) -> i32 {
        if url.len() <= start {
            return -1;
        }
        let ch0 = url[start];
        if ch0 == b':' {
            return (start + 1) as i32;
        }
        if ch0 == b'%' && url.len() > start + 2 && url[start + 1] == b'3' {
            let ch2 = url[start + 2];
            if ch2 == b'a' || ch2 == b'A' {
                return (start + 3) as i32;
            }
        }
        -1
    }

    // Go: tspath/path.go:169 GetEncodedRootLength

    // Go: tspath/path.go:243 GetRootLength

    // Go: tspath/path.go:251 GetDirectoryPath

    // Go: tspath/path.go:270 GetPathFromPathComponents

    // Go: tspath/path.go:283 NormalizeSlashes
    pub fn normalize_slashes(path: &str) -> String {
        path.replace('\\', "/")
    }

    // Go: tspath/path.go:287 reducePathComponents
    fn reduce_path_components(components: Vec<String>) -> Vec<String> {
        if components.is_empty() {
            return Vec::new();
        }
        let mut reduced = vec![components[0].clone()];
        for component in components.into_iter().skip(1) {
            if component.is_empty() {
                continue;
            }
            if component == "." {
                continue;
            }
            if component == ".." {
                if reduced.len() > 1 {
                    if reduced[reduced.len() - 1] != ".." {
                        reduced.pop();
                        continue;
                    }
                } else if !reduced[0].is_empty() {
                    continue;
                }
            }
            reduced.push(component);
        }
        reduced
    }

    // Go: tspath/path.go:394 GetNormalizedAbsolutePath

    // Go: tspath/path.go:515 simpleNormalizePath
    // PORT: Go returns `(string, bool)`; `None` is `("", false)`.
    fn simple_normalize_path(path: &str) -> Option<String> {
        // Most paths don't require normalization
        if !has_relative_path_segment(path) {
            return Some(path.to_string());
        }
        // Some paths only require cleanup of `/./` or leading `./`
        let simplified = path.replace("/./", "/");
        let trimmed = simplified.strip_prefix("./").unwrap_or(&simplified);
        if trimmed != path
            && !has_relative_path_segment(trimmed)
            && !(trimmed != simplified && trimmed.starts_with('/'))
        {
            // If we trimmed a leading "./" and the path now starts with "/", we changed the meaning
            return Some(trimmed.to_string());
        }
        None
    }

    // Go: tspath/path.go:532 hasRelativePathSegment
    // hasRelativePathSegment reports whether p contains ".", "..", "./", "../", "/.", "/..", "//", "/./", or "/../".
    fn has_relative_path_segment(p: &str) -> bool {
        let b = p.as_bytes();
        let n = b.len();
        if n == 0 {
            return false;
        }

        if p == "." || p == ".." {
            return true;
        }

        // Leading "./" OR "../"
        if b[0] == b'.' {
            if n >= 2 && b[1] == b'/' {
                return true;
            }
            // Leading "../"
            if n >= 3 && b[1] == b'.' && b[2] == b'/' {
                return true;
            }
        }
        // Trailing "/." OR "/.."
        if b[n - 1] == b'.' {
            if n >= 2 && b[n - 2] == b'/' {
                return true;
            }
            if n >= 3 && b[n - 2] == b'.' && b[n - 3] == b'/' {
                return true;
            }
        }

        // Now look for any `//` or `/./` or `/../`

        let mut prev_slash = false;
        let mut seg_len = 0; // length of current segment since last slash
        let mut dot_count: i32 = 0; // consecutive dots at start of the current segment; -1 => not only dots

        for &c in b {
            if c == b'/' {
                // "//"
                if prev_slash {
                    return true;
                }
                // "/./" or "/../"
                if (seg_len == 1 && dot_count == 1) || (seg_len == 2 && dot_count == 2) {
                    return true;
                }
                prev_slash = true;
                seg_len = 0;
                dot_count = 0;
                continue;
            }

            if c == b'.' {
                if dot_count >= 0 {
                    dot_count += 1;
                }
            } else {
                dot_count = -1;
            }
            seg_len += 1;
            prev_slash = false;
        }

        // Trailing "/." or "/.."
        (seg_len == 1 && dot_count == 1) || (seg_len == 2 && dot_count == 2)
    }

    // Go: tspath/path.go:600 NormalizePath

    // Go: tspath/path.go:612 GetCanonicalFileName

    // Go: tspath/path.go:674 ToFileNameLowerCase
    // PORT: Go `unicode.ToLower` maps one rune to one rune. Rust
    // `char::to_lowercase` can yield several; a multi-rune result keeps the
    // original rune, which matches Go for those runes. Go `strings.Map`
    // writes each byte that is not valid UTF-8 as U+FFFD (`go_map_runes`).

    // Go: tspath/path.go:723 ToPath
    // PORT: Go `tspath.Path` is a string type; this returns the string.

    // Go: tspath/path.go:733 RemoveTrailingDirectorySeparator
    fn remove_trailing_directory_separator(path: &str) -> &str {
        if has_trailing_directory_separator(path) {
            return &path[..path.len() - 1];
        }
        path
    }

    // Go: tspath/path.go:744 RemoveTrailingDirectorySeparators
    fn remove_trailing_directory_separators(path: &str) -> &str {
        let mut path = path;
        while has_trailing_directory_separator(path) {
            path = remove_trailing_directory_separator(path);
        }
        path
    }

    // Go: tspath/path.go:751 EnsureTrailingDirectorySeparator
    fn ensure_trailing_directory_separator(path: &str) -> String {
        if !has_trailing_directory_separator(path) {
            return format!("{path}/");
        }
        path.to_string()
    }

    // Go: tspath/path.go:765 GetPathComponentsRelativeTo

    // Go: tspath/path.go:809 GetRelativePathFromDirectory

    // Go: tspath/path.go:817 GetRelativePathFromFile

    // Go: tspath/path.go:876 GetBaseFileName
    // PERF: returns a slice of `path`. NormalizeSlashes swaps one byte for
    // one byte, and the base name has no separator, so the same byte range
    // of `path` holds the same text. Only a path with a backslash builds the
    // normalized copy.

    // The byte range of the Go GetBaseFileName result in the normalized `path`.

    // Go: tspath/path.go:902 GetAnyExtensionFromPath

    // Go: tspath/path.go:931 getAnyExtensionFromPathWorker
    // PORT: the Go equality comparer is `stringutil.GetStringEqualityComparer(ignoreCase)`.

    // Go: tspath/path.go:941 tryGetExtensionFromPath

    // Go: tspath/path.go:954 PathIsRelative
    pub fn path_is_relative(path: &str) -> bool {
        // True if path is ".", "..", or starts with "./", "../", ".\\", or "..\\".
        if path == "." || path == ".." {
            return true;
        }
        let b = path.as_bytes();
        if b.len() >= 2 && b[0] == b'.' && (b[1] == b'/' || b[1] == b'\\') {
            return true;
        }
        if b.len() >= 3 && b[0] == b'.' && b[1] == b'.' && (b[2] == b'/' || b[2] == b'\\') {
            return true;
        }
        false
    }

    // Go: tspath/path.go:974 EnsurePathIsNonModuleName

    // Go: tspath/path.go:981 IsExternalModuleNameRelative

    // Go: tspath/path.go:1095 FileExtensionIs
    pub fn file_extension_is(path: &str, extension: &str) -> bool {
        path.len() > extension.len() && path.ends_with(extension)
    }

    // Go: tspath/path.go:1139 HasExtension

    // Go: tspath/extension.go:39 ExtensionIsTs
    pub fn extension_is_ts(ext: &str) -> bool {
        ext == EXTENSION_TS
            || ext == EXTENSION_TSX
            || ext == EXTENSION_DTS
            || ext == EXTENSION_MTS
            || ext == EXTENSION_DMTS
            || ext == EXTENSION_CTS
            || ext == EXTENSION_DCTS
            || ext.len() >= 7 && &ext[..3] == ".d." && &ext[ext.len() - 3..] == ".ts"
    }

    // Go: tspath/extension.go TryGetExtensionFromPath
    pub fn try_get_extension_from_path(p: &str) -> &'static str {
        for ext in EXTENSIONS_TO_REMOVE {
            if file_extension_is(p, ext) {
                return ext;
            }
        }
        ""
    }

    // Go: tspath/extension.go RemoveExtension
    pub fn remove_extension<'a>(path: &'a str, extension: &str) -> &'a str {
        &path[..path.len() - extension.len()]
    }

    // Go: tspath/extension.go FileExtensionIsOneOf
    fn file_extension_is_one_of(path: &str, extensions: &[&str]) -> bool {
        extensions.iter().any(|ext| file_extension_is(path, ext))
    }

    // Go: tspath/extension.go TryExtractTSExtension
    pub fn try_extract_ts_extension(file_name: &str) -> &'static str {
        for ext in SUPPORTED_TS_EXTENSIONS_FOR_EXTRACT_EXTENSION {
            if file_extension_is(file_name, ext) {
                return ext;
            }
        }
        ""
    }

    // Go: tspath/extension.go HasTSFileExtension
    pub fn has_ts_file_extension(path: &str) -> bool {
        file_extension_is_one_of(path, &SUPPORTED_TS_EXTENSIONS_FLAT)
    }

    // Go: tspath/extension.go IsDeclarationFileName

    // Go: tspath/extension.go GetDeclarationFileExtension
    // PERF: returns a slice of `file_name` or a constant, not a copy.
}

/// Private copies of Go `core` helpers used by this unit.
mod core_p17 {
    use crate::prelude::*;

    // Go: core/nodemodules.go UnprefixedNodeCoreModules
    const UNPREFIXED_NODE_CORE_MODULES: [&str; 54] = [
        "assert",
        "assert/strict",
        "async_hooks",
        "buffer",
        "child_process",
        "cluster",
        "console",
        "constants",
        "crypto",
        "dgram",
        "diagnostics_channel",
        "dns",
        "dns/promises",
        "domain",
        "events",
        "fs",
        "fs/promises",
        "http",
        "http2",
        "https",
        "inspector",
        "inspector/promises",
        "module",
        "net",
        "os",
        "path",
        "path/posix",
        "path/win32",
        "perf_hooks",
        "process",
        "punycode",
        "querystring",
        "readline",
        "readline/promises",
        "repl",
        "stream",
        "stream/consumers",
        "stream/promises",
        "stream/web",
        "string_decoder",
        "sys",
        "timers",
        "timers/promises",
        "tls",
        "trace_events",
        "tty",
        "url",
        "util",
        "util/types",
        "v8",
        "vm",
        "wasi",
        "worker_threads",
        "zlib",
    ];

    // Go: core/nodemodules.go ExclusivelyPrefixedNodeCoreModules
    const EXCLUSIVELY_PREFIXED_NODE_CORE_MODULES: [&str; 5] = [
        "node:quic",
        "node:sea",
        "node:sqlite",
        "node:test",
        "node:test/reporters",
    ];

    // Go: core/nodemodules.go NodeCoreModules
    // PORT: Go builds a map once (each unprefixed name plus its "node:" form,
    // plus the exclusively prefixed names). This checks membership directly.
    pub fn is_node_core_module(name: &str) -> bool {
        let unprefixed = name.strip_prefix("node:").unwrap_or(name);
        UNPREFIXED_NODE_CORE_MODULES.contains(&unprefixed)
            || EXCLUSIVELY_PREFIXED_NODE_CORE_MODULES.contains(&name)
    }

    // Go: core/core.go:724 ShouldRewriteModuleSpecifier

    // Go: core/pattern.go FindBestPatternMatch
    // PORT: `ast.PatternAmbientModule.Pattern` is stored as a prefix and a
    // suffix. Ambient module patterns always contain a star, so Go
    // `StarIndex` is the prefix length and `Pattern.Matches` checks
    // `len(candidate) >= len(Text)-1` plus the prefix and suffix.

    // Go: core/pattern.go:22 Pattern.Matches
    // PORT: for `ast.PatternAmbientModule.Pattern`, which always has a star
    // (see `find_best_pattern_match`). `candidate` is the Go bytes of the
    // candidate (`go_string_bytes`), so a caller that tests many patterns
    // makes them once.
    // PERF: a first or last byte that differs rejects the pattern before
    // the `memcmp` call of `starts_with` or `ends_with`.
}

/// Private copy of Go `module.GetResolutionDiagnostic`.
mod module_p17 {
    use super::tspath_p17::*;
    use crate::prelude::*;

    // Go: module/util.go:125 GetResolutionDiagnostic
}

#[cfg(test)]
mod pattern_match_tests {
    use super::core_p17::pattern_matches_go_bytes;
    use crate::prelude::*;

    /// `pattern_matches_go_bytes` gives Go `Pattern.Matches`
    /// (core/pattern.go:22) on the Go bytes: `len(candidate) >= len(Text)-1`,
    /// `strings.HasPrefix` and `strings.HasSuffix`. The texts include empty
    /// affixes, affixes that overlap in a short candidate, and port forms
    /// with invalid byte units, where a byte suffix can match inside a char.
    #[test]
    fn pattern_matches_as_go_on_go_bytes() {
        let invalid = |bytes: &[u8]| go_string_from_bytes(bytes.to_vec());
        let patterns = [
            ("", ".svg"),
            ("virtual:", ""),
            ("ab", "ba"),
            ("", ""),
            ("", &invalid(&[0xA9])),
            (&invalid(&[0xC3]), ""),
            ("x", &invalid(&[0xFF, b'z'])),
        ];
        let candidates = [
            "a.svg",
            ".svg",
            "svg",
            "virtual:x",
            "virtual",
            "aba",
            "abba",
            "ab-ba",
            "",
            "é",
            "xé",
            &invalid(&[b'x', 0xFF, b'z']),
            &invalid(&[0xC3]),
        ];
        let mut got = Vec::new();
        let mut want = Vec::new();
        for (prefix, suffix) in &patterns {
            let value = PatternAmbientModule {
                pattern_prefix: prefix.to_string(),
                pattern_suffix: suffix.to_string(),
                symbol: SymbolId::NIL,
            };
            for candidate in &candidates {
                let (c, p, s) = (
                    go_string_bytes(candidate),
                    go_string_bytes(prefix),
                    go_string_bytes(suffix),
                );
                want.push(c.len() >= p.len() + s.len() && c.starts_with(&p) && c.ends_with(&s));
                got.push(pattern_matches_go_bytes(&value, &c));
            }
        }
        assert_eq!(got, want);
        // `é` (C3 A9) ends with the byte A9, as in Go.
        assert!(got[4 * candidates.len() + 9]);
        assert_eq!(got.iter().filter(|&&m| m).count(), 23);
    }
}
