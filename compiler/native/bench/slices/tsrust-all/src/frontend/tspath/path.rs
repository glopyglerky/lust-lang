//! Port of tspath/path.go and tspath/ignoredpaths.go.

use crate::frontend::prelude::*;

use crate::gostd::unicode;
use std::borrow::Cow;

// Go: tspath/path.go:14 Path
// PORT: Go `type Path string`. A newtype keeps the Go method set
// (`GetDirectoryPath`, `ContainsPath`, ...). `Deref<Target = str>` and
// `Borrow<str>` stand in for Go `string(p)`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Path(pub String);

impl Path {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl std::ops::Deref for Path {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl std::borrow::Borrow<str> for Path {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

// Internally, we represent paths as strings with '/' as the directory separator.
// When we make system calls (eg: LanguageServiceHost.getDirectory()),
// we expect the host to correctly handle paths in our specified format.
pub const DIRECTORY_SEPARATOR: u8 = b'/';
const URL_SCHEME_SEPARATOR: &str = "://";

// ---------------------------------------------------------------------------
// stringutil/compare.go helpers used by tspath.
// PORT: stringutil is not a unit of this wave. These are private copies of
// the Go functions that tspath calls, so no other unit's names are claimed.
// ---------------------------------------------------------------------------

// PORT: simple uppercase. A multi-rune full uppercase (for example U+00DF)
// has no simple mapping in UnicodeData, so the rune stays the same.

// PORT: simple case fold key. Two runes are in the same Go `unicode.SimpleFold`
// orbit when their keys are equal. The key is lower(upper(c)) with simple
// mappings. U+0131 (dotless i) has only a Turkic fold entry, so its orbit is
// itself, like Go. U+0130 stays itself because it has no simple lowercase
// other than through the full mapping. Rust and Go can use different Unicode
// versions; this matters only for runes added between those versions.

// Go: stringutil/compare.go:9 EquateStringCaseInsensitive
// PORT: Go `strings.EqualFold`. Each rune pair must be equal, ASCII case
// equal, or in the same simple fold orbit (see `simple_fold_key`).

// Go: stringutil/compare.go:15 EquateStringCaseSensitive
fn equate_string_case_sensitive(a: &str, b: &str) -> bool {
    a == b
}

// Go: stringutil/compare.go:19 GetStringEqualityComparer

// Go: stringutil/compare.go:34 CompareStringsCaseInsensitive

// Go: stringutil/compare.go:63 CompareStringsCaseSensitive
// PORT: Go compares bytes; the strings are port forms (see
// `scanner_util::compare_go_bytes`).

// Go: stringutil/compare.go:67 GetStringComparer

// PORT: Go `strings.LastIndex(s, "/")` / `LastIndexByte` as a signed index.
fn last_index_byte(s: &str, c: u8) -> isize {
    s.as_bytes()
        .iter()
        .rposition(|&b| b == c)
        .map_or(-1, |i| i as isize)
}

//// Path Tests

// Go: tspath/path.go:27 isAnyDirectorySeparator
// Determines whether a byte corresponds to `/` or `\`.
fn is_any_directory_separator(char: u8) -> bool {
    char == b'/' || char == b'\\'
}

// Go: tspath/path.go:32 IsUrl
// Determines whether a path starts with a URL scheme (e.g. starts with `http://`, `ftp://`, `file://`, etc.).

// Go: tspath/path.go:38 IsRootedDiskPath
// Determines whether a path is an absolute disk path (e.g. starts with `/`, or a dos path
// like `c:`, `c:\` or `c:/`).

// Go: tspath/path.go:43 IsDiskPathRoot
// Determines whether a path consists only of a path root.

// Go: tspath/path.go:50 IsDynamicFileName
// IsDynamicFileName returns true if the file name represents a dynamic/virtual file
// that doesn't exist on disk (e.g., untitled files with paths like "^/untitled/...").
pub fn is_dynamic_file_name(file_name: &str) -> bool {
    file_name.starts_with("^/")
}

// Go: tspath/path.go:67 PathIsAbsolute
// Determines whether a path starts with an absolute path component (i.e. `/`, `c:/`, `file://`, etc.).

// Go: tspath/path.go:71 HasTrailingDirectorySeparator
pub fn has_trailing_directory_separator(path: &str) -> bool {
    !path.is_empty() && is_any_directory_separator(path.as_bytes()[path.len() - 1])
}

// Go: tspath/path.go:91 CombinePaths
// Combines paths. If a path is absolute, it replaces any previous path. Relative paths are not simplified.
// PORT: Go writes into one `strings.Builder` and "sets" the result by moving
// `start`. Building the current result directly gives the same string.

// Go: tspath/path.go:134 GetPathComponents

// Go: tspath/path.go:139 pathComponents

// Go: tspath/path.go:148 IsVolumeCharacter

// Go: tspath/path.go:152 getFileUrlVolumeSeparatorEnd
fn get_file_url_volume_separator_end(url: &str, start: usize) -> i32 {
    let url = url.as_bytes();
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
// PORT: returns `usize` because the result is never negative and is used to slice.

// Go: tspath/path.go:251 GetDirectoryPath
// PERF (pgoedit1): one string, not two. `normalize_slashes_cow` borrows a
// path that has no backslash, as Go's `NormalizeSlashes` returns its input.
// memchr and memrchr pick their SIMD code at run time. The byte loops they
// replace were vectorized or not by the PGO profile alone: in the R175
// release build the language server, whose loader resolves each import name
// itself, paid about 5x the instructions here (state note
// edbisect1-2026-10-06, `scripts/state history --kind note`).

impl Path {
    // Go: tspath/path.go:266 Path.GetDirectoryPath
}

// Go: tspath/path.go:270 GetPathFromPathComponents

// Go: tspath/path.go:283 NormalizeSlashes
pub fn normalize_slashes(path: &str) -> String {
    normalize_slashes_cow(path).into_owned()
}

/// `normalize_slashes` that borrows `path` when it has no backslash.
// PORT: Go's `strings.ReplaceAll` returns its input when nothing matches,
// so Go makes no copy there either.
pub fn normalize_slashes_cow(path: &str) -> Cow<'_, str> {
    if crate::memchr::memchr(b'\\', path.as_bytes()).is_some() {
        Cow::Owned(path.replace('\\', "/"))
    } else {
        Cow::Borrowed(path)
    }
}

// Go: tspath/path.go:287 reducePathComponents
fn reduce_path_components(components: Vec<String>) -> Vec<String> {
    if components.is_empty() {
        return Vec::new();
    }
    let mut components = components.into_iter();
    let mut reduced = vec![components.next().unwrap_or_default()];
    for component in components {
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

// Go: tspath/path.go:323 ResolvePath
// Combines and resolves paths. If a path is absolute, it replaces any previous path. Any
// `.` and `..` path components are resolved. Trailing directory separators are preserved.

// Go: tspath/path.go:333 ResolveTripleslashReference

// Go: tspath/path.go:341 GetNormalizedPathComponents

// Go: tspath/path.go:346 getNormalizedPathComponentsFromCombined

// Go: tspath/path.go:388 GetNormalizedAbsolutePathWithoutRoot

// Go: tspath/path.go:394 GetNormalizedAbsolutePath

// Go: tspath/path.go:515 simpleNormalizePath
// PORT: Go `(string, bool)` becomes `Option<String>`.

// Go: tspath/path.go:532 hasRelativePathSegment
// hasRelativePathSegment reports whether p contains ".", "..", "./", "../", "/.", "/..", "//", "/./", or "/../".


// Go: tspath/path.go:532 hasRelativePathSegment (the debug check of
// `has_relative_path_segment`).
fn has_relative_path_segment_go(p: &str) -> bool {
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

// Go: tspath/path.go:629 TrimFilePathPrefix (tsgo#4900)
// TrimFilePathPrefix removes prefix from the start of path, honoring
// useCaseSensitiveFileNames the same way GetCanonicalFileName does. It returns
// the remainder of path if path starts with prefix.
//
// This must not slice path using len(prefix): case-folding (as performed by
// GetCanonicalFileName) can change a string's UTF-8 byte length without
// changing its rune count (e.g. the Kelvin sign '\u212A' case-folds to the
// single-byte 'k'), so path and prefix can disagree in byte length even when
// one is (a case-insensitive match for) a prefix of the other.
// PORT: Go returns `path, false` when path does not start with prefix; this
// returns `None`. `path` and `prefix` are port forms (see
// `scanner_util::GO_STRING_MARKER`): the test and the cut use the Go bytes,
// and the remainder is in the value form (`go_slice`).

// Go: tspath/path.go:642 trimRuneCount (tsgo#4900)
// trimRuneCount returns the suffix of s after skipping up to runeCount runes,
// clamping to the end of s if it has fewer runes than runeCount.
// PORT: Go decodes the bytes of `s`, so a byte that is not valid UTF-8 is one
// rune. This decodes the Go bytes of the port form the same way.

// PORT: Go `utf8.RuneCountInString` on the Go bytes of the port form `s`:
// each byte that is not valid UTF-8 is one rune.

// Go: tspath/path.go:674 ToFileNameLowerCase
// We convert the file names to lower case as key for file name on case insensitive file system
// While doing so we need to handle special characters (eg \u0130) to ensure that we dont convert
// it to lower case, fileName with its lowercase form can exist along side it.
// Handle special characters and make those case sensitive instead
//
// Because \u0130 is special where in its lowercase character has its own
// upper case form we cant convert its case.
// Rest special characters are either already in lower case format or
// they have corresponding upper case character so they dont need special handling
// PORT: the Go `unsafe.String` fast path is a plain ASCII lowercase copy.
// `file_name` is a port form (see `scanner_util::GO_STRING_MARKER`). Go
// `strings.Map` writes each byte that is not valid UTF-8 as U+FFFD, and
// `go_map_runes` does the same.

// Go: tspath/path.go:723 ToPath

// Go: tspath/path.go:733 RemoveTrailingDirectorySeparator
pub fn remove_trailing_directory_separator(path: &str) -> &str {
    if has_trailing_directory_separator(path) {
        return &path[..path.len() - 1];
    }
    path
}

impl Path {
    // Go: tspath/path.go:740 Path.RemoveTrailingDirectorySeparator
    pub fn remove_trailing_directory_separator(&self) -> Path {
        Path(remove_trailing_directory_separator(&self.0).to_string())
    }
}

// Go: tspath/path.go:744 RemoveTrailingDirectorySeparators
pub fn remove_trailing_directory_separators(path: &str) -> &str {
    let mut path = path;
    while has_trailing_directory_separator(path) {
        path = remove_trailing_directory_separator(path);
    }
    path
}

// Go: tspath/path.go:751 EnsureTrailingDirectorySeparator
pub fn ensure_trailing_directory_separator(path: &str) -> String {
    if !has_trailing_directory_separator(path) {
        return format!("{path}/");
    }

    path.to_string()
}

impl Path {
    // Go: tspath/path.go:759 Path.EnsureTrailingDirectorySeparator
    pub fn ensure_trailing_directory_separator(&self) -> Path {
        Path(ensure_trailing_directory_separator(&self.0))
    }
}

//// Relative Paths

// Go: tspath/path.go:765 GetPathComponentsRelativeTo

// Go: tspath/path.go:809 GetRelativePathFromDirectory

// Go: tspath/path.go:817 GetRelativePathFromFile

// Go: tspath/path.go:821 ConvertToRelativePath

// Go: tspath/path.go:829 GetRelativePathToDirectoryOrUrl

// Go: tspath/path.go:876 GetBaseFileName
// Gets the portion of a path following the last (non-terminal) separator (`/`).
// Semantics align with NodeJS's `path.basename` except that we support URL's as well.
// If the base name has any one of the provided extensions, it is removed.

// Go: tspath/path.go:902 GetAnyExtensionFromPath
// Gets the file extension for a path.
// If extensions are provided, gets the file extension for a path, provided it is one of the provided extensions.

// Go: tspath/path.go:917 GetLongestExtensionFromPath (tsgo#4712)

// Go: tspath/path.go:931 getAnyExtensionFromPathWorker
fn get_any_extension_from_path_worker(
    path: &str,
    extensions: &[&str],
    string_equality_comparer: fn(&str, &str) -> bool,
) -> String {
    for extension in extensions {
        let result = try_get_extension_from_path_worker(path, extension, string_equality_comparer);
        if !result.is_empty() {
            return result;
        }
    }
    String::new()
}

// Go: tspath/path.go:941 tryGetExtensionFromPath
// PORT: renamed to `try_get_extension_from_path_worker`. The exported Go
// `TryGetExtensionFromPath` (extension.go) has the same snake name.
fn try_get_extension_from_path_worker(
    path: &str,
    extension: &str,
    string_equality_comparer: fn(&str, &str) -> bool,
) -> String {
    let extension = if !extension.starts_with('.') {
        format!(".{extension}")
    } else {
        extension.to_string()
    };
    let pb = path.as_bytes();
    if pb.len() >= extension.len() && pb[pb.len() - extension.len()] == b'.' {
        let path_extension = &path[path.len() - extension.len()..];
        if string_equality_comparer(path_extension, &extension) {
            return path_extension.to_string();
        }
    }
    String::new()
}

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
// EnsurePathIsNonModuleName ensures a path is either absolute (prefixed with `/` or `c:`) or dot-relative (prefixed
// with `./` or `../`) so as not to be confused with an unprefixed module name.

// Go: tspath/path.go:981 IsExternalModuleNameRelative

// Go: tspath/path.go:988 ComparePathsOptions
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ComparePathsOptions {
    pub use_case_sensitive_file_names: bool,
    pub current_directory: String,
}

impl ComparePathsOptions {
    // Go: tspath/path.go:993 ComparePathsOptions.GetComparer

    // Go: tspath/path.go:997 ComparePathsOptions.getEqualityComparer
}

// Go: tspath/path.go:1001 ComparePaths

// Go: tspath/path.go:1046 ComparePathsCaseSensitive

// Go: tspath/path.go:1050 ComparePathsCaseInsensitive

// Go: tspath/path.go:1054 ContainsPath

impl Path {
    // Go: tspath/path.go:1088 Path.ContainsPath
    // ContainsPath checks whether child is contained within or equal to p.
    // Since Path values are already rooted, reduced, and case-canonicalized,
    // this is a simple string prefix check.
}

// Go: tspath/path.go:1095 FileExtensionIs
pub fn file_extension_is(path: &str, extension: &str) -> bool {
    path.len() > extension.len() && path.ends_with(extension)
}

// Go: tspath/path.go:1101 ForEachAncestorDirectoryStoppingAtGlobalCache
// Calls `callback` on `directory` and every ancestor directory it has, returning the first defined result.
// Stops at global cache location

// Go: tspath/path.go:1116 ForEachAncestorDirectory
// PORT: the Go zero value of T is `T::default()`.

impl Path {
    // Go: tspath/path.go:1133 (Path).ForEachAncestorDirectory (ts#63902)
}

// Go: tspath/path.go:1139 HasExtension

// Go: tspath/path.go:1143 SplitVolumePath
// PORT: Go `(volume, rest, ok)`. The volume is two ASCII bytes, so
// `strings.ToLower` is an ASCII lowercase.

// Go: tspath/path.go:1158 GetCommonParents
// GetCommonParents returns the smallest set of directories that are parents of all paths with
// at least `minComponents` directory components. Any path that has fewer than `minComponents` directory components
// will be returned in the second return value.
// PORT: a Go nil slice or nil map is an empty `Vec` or set. Go map iteration
// order is random, so callers cannot depend on the set order.

// Go: tspath/path.go:1197 getCommonParentsWorker
// ts#64493: each result gets its own copy of the group head (Go no longer
// appends into the head's backing array). The function is used only by
// project watching, which is out of scope for the frontend.

// Go: tspath/path.go:1257 StartsWithDirectory

// Go: tspath/path.go:1271 CompareNumberOfDirectorySeparators

// Go: tspath/ignoredpaths.go:5 ignoredPaths
const IGNORED_PATHS: [&str; 3] = ["/node_modules/.", "/.git", ".#"];

// Go: tspath/ignoredpaths.go:11 ContainsIgnoredPath
pub fn contains_ignored_path(path: &str) -> bool {
    for pattern in IGNORED_PATHS {
        if path.contains(pattern) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod relative_segment_tests {
    use super::{
        has_dot_or_empty_segment, has_relative_path_segment_go, is_normalized_path, normalize_path,
        to_path,
    };

    /// `is_normalized_path` is true only for a name that `normalize_path`
    /// keeps, and the `to_path` fast path for it gives the path of the slow
    /// path, for every name of up to 6 bytes over '.', '/', '\\', ':', 'a'
    /// and 'B' (so rooted, drive-rooted and relative names), on both kinds
    /// of file system.
    #[test]
    fn normalized_path_fast_paths_match() {
        let slow_to_path = |name: &str, base: &str, case_sensitive: bool| {
            let normal = if super::is_rooted_disk_path(name) {
                normalize_path(name)
            } else {
                super::get_normalized_absolute_path(name, base)
            };
            super::get_canonical_file_name(&normal, case_sensitive)
        };
        let mut level = vec![String::new()];
        for _ in 0..6 {
            level = level
                .iter()
                .flat_map(|path| ['.', '/', '\\', ':', 'a', 'B'].map(|c| format!("{path}{c}")))
                .collect();
            for path in &level {
                if is_normalized_path(path) {
                    assert_eq!(normalize_path(path), *path, "{path}");
                }
                for case_sensitive in [true, false] {
                    assert_eq!(
                        to_path(path, "/base/Dir", case_sensitive).0,
                        slow_to_path(path, "/base/Dir", case_sensitive),
                        "{path} {case_sensitive}"
                    );
                }
            }
        }
    }

    /// The fast `has_relative_path_segment` gives Go's answer for every path
    /// of up to 8 bytes over '.', '/' and 'a'.
    #[test]
    fn has_relative_path_segment_matches_go() {
        let mut level = vec![String::new()];
        for _ in 0..8 {
            level = level
                .iter()
                .flat_map(|path| ['.', '/', 'a'].map(|c| format!("{path}{c}")))
                .collect();
            for path in &level {
                assert_eq!(
                    has_dot_or_empty_segment(path.as_bytes()),
                    has_relative_path_segment_go(path),
                    "{path}"
                );
            }
        }
    }
}

#[cfg(test)]
mod unicode_case_tests {
    use super::{compare_strings_case_insensitive, to_path};

    // Go `ToPath` on a case-insensitive host (macOS, Windows) lowers with
    // `unicode.ToLower` (tspath/path.go:674 ToFileNameLowerCase). The pin N
    // oracle is go1.27.1 (Unicode 17.0.0), where U+A7CB lowers to U+0264 and
    // U+10D50 to U+10D70. go1.26.8 (Unicode 15.0.0) kept them.
    #[test]
    fn to_path_lowers_unicode_17_letters() {
        assert_eq!(
            to_path("/Proj/\u{A7CB}\u{10D50}.ts", "/", false).0,
            "/proj/\u{264}\u{10D70}.ts"
        );
        assert_eq!(
            to_path("/Proj/\u{A7CB}.ts", "/", true).0,
            "/Proj/\u{A7CB}.ts"
        );
        assert_eq!(
            compare_strings_case_insensitive("/a/\u{A7CB}", "/a/\u{264}"),
            0
        );
    }
}

#[cfg(test)]
mod directory_path_tests {
    use super::{
        get_directory_path, get_root_length, normalize_slashes, normalize_slashes_cow,
        remove_trailing_directory_separator,
    };
    use std::borrow::Cow;

    /// Go's `GetDirectoryPath` (tspath/path.go:251) as Go writes it: copy,
    /// then a second string.
    fn go_get_directory_path(path: &str) -> String {
        let path = path.replace('\\', "/");
        let root_length = get_root_length(&path);
        if root_length == path.len() {
            return path;
        }
        let path = remove_trailing_directory_separator(&path);
        let last_slash = path.rfind('/').map_or(-1, |i| i as isize);
        path[..(root_length as isize).max(last_slash) as usize].to_string()
    }

    /// `get_directory_path` gives Go's answer for every name of up to 5
    /// bytes over '/', '\\', ':', '.', 'a' and 'c', alone and after disk,
    /// UNC and URL roots, with and without backslashes.
    #[test]
    fn directory_path_matches_go() {
        let roots = [
            "",
            "/",
            "c:",
            "c:\\",
            "//server/",
            "\\\\server\\share\\",
            "file:///",
            "file:///c:/",
            "file://server/",
            "http://server/",
            "https://h/\u{e4}\\\u{fc}/",
        ];
        let mut level = vec![String::new()];
        let mut names = vec![String::new()];
        for _ in 0..5 {
            level = level
                .iter()
                .flat_map(|path| ['/', '\\', ':', '.', 'a', 'c'].map(|c| format!("{path}{c}")))
                .collect();
            names.extend(level.iter().cloned());
        }
        for root in roots {
            for name in &names {
                let path = format!("{root}{name}");
                assert_eq!(
                    get_directory_path(&path),
                    go_get_directory_path(&path),
                    "{path:?}"
                );
            }
        }
    }

    /// `normalize_slashes_cow` borrows a path without a backslash and
    /// replaces each backslash otherwise.
    #[test]
    fn normalize_slashes_borrows_without_backslash() {
        for path in [
            "",
            "a",
            "/a/b.ts",
            "c:/a",
            "file:///c:/a",
            "http://server/\u{e4}",
        ] {
            assert!(matches!(normalize_slashes_cow(path), Cow::Borrowed(p) if p == path));
            assert_eq!(normalize_slashes(path), path);
        }
        for (path, want) in [
            ("\\", "/"),
            ("a\\b", "a/b"),
            ("c:\\a\\", "c:/a/"),
            ("\\\\server\\share", "//server/share"),
            ("file:///c:\\\u{e4}\\b", "file:///c:/\u{e4}/b"),
        ] {
            assert!(matches!(normalize_slashes_cow(path), Cow::Owned(ref p) if p == want));
            assert_eq!(normalize_slashes(path), want);
        }
    }
}
