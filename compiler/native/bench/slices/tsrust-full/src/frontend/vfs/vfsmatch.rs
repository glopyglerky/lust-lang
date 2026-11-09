use crate::frontend::prelude::*;
use std::borrow::Cow;

// This file ports vfs/vfsmatch/vfsmatch.go and stringer_generated.go.
// It implements the glob matching algorithm specified in MATCHING_ALGORITHM.md.
// PORT: Go slices strings at byte offsets that can be inside a UTF-8
// sequence. Rust `str` cannot be sliced there, so the matcher compares byte
// slices and decodes runes like Go `utf8.DecodeRuneInString` (see
// `decode_rune`).

// Go: vfs/vfsmatch/vfsmatch.go:19 Usage
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Usage {
    Files,
    Directories,
    Exclude,
}

// Go: vfs/vfsmatch/stringer_generated.go:20 Usage.String
// PORT: a Rust enum has no out-of-range values, so the `Usage(%d)` branch
// is not needed.
impl std::fmt::Display for Usage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Usage::Files => "Files",
            Usage::Directories => "Directories",
            Usage::Exclude => "Exclude",
        };
        f.write_str(name)
    }
}

// Go: vfs/vfsmatch/vfsmatch.go:28 UnlimitedDepth
// PORT: Go `math.MaxInt`. Go `int` is `i32` in this port.
/// Pass as the depth argument to indicate there is no depth limit.
pub const UNLIMITED_DEPTH: i32 = i32::MAX;

// Go: vfs/vfsmatch/vfsmatch.go:30 ReadDirectory

// Go: vfs/vfsmatch/vfsmatch.go:37 IsImplicitGlob

// Go: vfs/vfsmatch/vfsmatch.go:41 wildcardCharCodes
const WILDCARD_CHAR_CODES: [char; 2] = ['*', '?'];

// Go: vfs/vfsmatch/vfsmatch.go:43 getIncludeBasePath

// Go: vfs/vfsmatch/vfsmatch.go:57 getBasePaths

// Go: vfs/vfsmatch/vfsmatch.go:98 globPattern
/// A compiled glob pattern for matching file paths without regex.
#[derive(Clone, Debug, Default)]
struct GlobPattern {
    /// path segments to match (e.g., ["src", "**", "*.ts"])
    components: Vec<Component>,
    /// exclude patterns have different matching rules
    is_exclude: bool,
    case_sensitive: bool,
    /// for "files" patterns, exclude .min.js by default
    exclude_min_js: bool,
    /// PERF (cfgwalk1): bit `i` is set when component `i` is `**` (see
    /// `prefix_states`).
    double_asterisks: u64,
}

// Go: vfs/vfsmatch/vfsmatch.go:107 component
/// A single path segment in a glob pattern.
/// Examples: "src" (literal), "*" (wildcard), "*.ts" (wildcard), "**" (recursive)
#[derive(Clone, Debug)]
struct Component {
    kind: ComponentKind,
    /// for Literal: the exact string to match
    literal: String,
    /// PERF (cfgwalk1): the Go bytes of `literal` (see
    /// `scanner_util::GO_STRING_MARKER`), made once at compile time.
    literal_bytes: Vec<u8>,
    /// for Wildcard: parsed wildcard pattern
    segments: Vec<Segment>,
    /// Include patterns with wildcards skip common package folders (node_modules, etc.)
    skip_package_folders: bool,
}

// Go: vfs/vfsmatch/vfsmatch.go:115 componentKind
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ComponentKind {
    /// exact match (e.g., "src")
    Literal,
    /// contains * or ? (e.g., "*.ts")
    Wildcard,
    /// ** matches zero or more directories
    DoubleAsterisk,
}

// Go: vfs/vfsmatch/vfsmatch.go:125 segment
/// A piece of a wildcard component.
/// Example: "*.ts" becomes [Star, Literal(".ts")]
#[derive(Clone, Debug)]
struct Segment {
    kind: SegmentKind,
    /// only for Literal
    literal: String,
    /// PERF (cfgwalk1): the Go bytes of `literal`, made once.
    literal_bytes: Vec<u8>,
}

// Go: vfs/vfsmatch/vfsmatch.go:130 segmentKind
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SegmentKind {
    /// exact text
    Literal,
    /// * matches any chars except /
    Star,
    /// ? matches single char except /
    Question,
}

// Go: vfs/vfsmatch/vfsmatch.go:140 compileGlobPattern

// Go: vfs/vfsmatch/vfsmatch.go:171 parseComponent

// Go: vfs/vfsmatch/vfsmatch.go:186 parseSegments

impl GlobPattern {
    // Go: vfs/vfsmatch/vfsmatch.go:219 (*globPattern).matches

    // Go: vfs/vfsmatch/vfsmatch.go:225 (*globPattern).matchesParts

    // Go: vfs/vfsmatch/vfsmatch.go:230 (*globPattern).matchesPrefixParts

    // Go: vfs/vfsmatch/vfsmatch.go:236 (*globPattern).matchPathParts

    // Go: vfs/vfsmatch/vfsmatch.go:283 (*globPattern).patternSatisfied

    // PERF (cfgwalk1): `match_path_parts` as a set of states, so that the
    // parts of a directory are matched once for all its entries (see
    // `GlobVisitor::visit`). Go matches every part of the absolute path again
    // for each entry. A state is a component index, bit `i` of a `u64`; bit
    // `len` is the state after the last component. `match_path_parts` is an
    // OR over its branches with no side effects, so the states give the same
    // result:
    // - `**` at `i` tries `i + 1` on the same part (the closure), and keeps
    //   `i` on the next part unless an include meets a hidden or package
    //   folder part;
    // - a literal or wildcard at `i` goes to `i + 1` when it matches the part;
    // - a part left at state `len` ends that branch with
    //   `is_exclude && !prefix_only` (`overrun`);
    // - at the end of the path a state `i` gives `prefix_only ||
    //   pattern_satisfied(i)`.
    // Patterns of 64 or more components use `match_path_parts`.


    /// The states and the `**` states that they reach on the same part.
    fn state_closure(&self, mut live: u64) -> u64 {
        loop {
            let reached = ((live & self.double_asterisks) << 1) & !live;
            if reached == 0 {
                return live;
            }
            live |= reached;
        }
    }



}

// PERF (cfgwalk1): the parts of `prefix`, a path that ends in '/', as
// `next_path_part_parts(prefix, suffix, ..)` gives them before the suffix,
// with their Go bytes.

// PERF (cfgwalk1): the states of one pattern after the parts of a directory
// path (see `GlobPattern::prefix_states`).
#[derive(Clone, Copy, Debug)]
struct PrefixStates {
    live: u64,
    /// A branch had a part left after the last component.
    overrun: bool,
}

// Go: vfs/vfsmatch/vfsmatch.go:294 nextPathPartSingle

// Go: vfs/vfsmatch/vfsmatch.go:314 nextPathPartParts

impl GlobPattern {
    // Go: vfs/vfsmatch/vfsmatch.go:361 (*globPattern).matchWildcard

    // Go: vfs/vfsmatch/vfsmatch.go:382 (*globPattern).matchSegments

    // Go: vfs/vfsmatch/vfsmatch.go:433 (*globPattern).shouldIncludeMinJs

    // Go: vfs/vfsmatch/vfsmatch.go:451 (*globPattern).hasMinJsSuffix

    // Go: vfs/vfsmatch/vfsmatch.go:463 (*globPattern).patternMentionsMinSuffix

    // Go: vfs/vfsmatch/vfsmatch.go:480 (*globPattern).stringsEqual
}

// Go: vfs/vfsmatch/vfsmatch.go:487 isHiddenPath
/// Checks if a path component is hidden (starts with dot).
fn is_hidden_path(name: &str) -> bool {
    is_hidden_path_bytes(name.as_bytes())
}

// PORT: byte form of `isHiddenPath` for `matchWildcard`, which works on bytes.
fn is_hidden_path_bytes(name: &[u8]) -> bool {
    !name.is_empty() && name[0] == b'.'
}

// Go: vfs/vfsmatch/vfsmatch.go:492 isPackageFolder

// Go: vfs/vfsmatch/vfsmatch.go:504 ensureTrailingSlash

// PORT: Go `utf8.DecodeRuneInString`. An invalid or truncated sequence
// gives U+FFFD with width 1. The input is never empty at the call sites.
fn decode_rune(s: &[u8]) -> (char, usize) {
    // PERF (cfgwalk1): an ASCII byte is its own rune, as in Go's fast path.
    if let Some(&b) = s.first()
        && b < 0x80
    {
        return (b as char, 1);
    }
    let head = &s[..s.len().min(4)];
    let valid = match std::str::from_utf8(head) {
        Ok(v) => v,
        // `valid_up_to` marks the valid UTF-8 prefix.
        Err(e) => std::str::from_utf8(&head[..e.valid_up_to()]).unwrap_or(""),
    };
    match valid.chars().next() {
        Some(c) => (c, c.len_utf8()),
        None => (char::REPLACEMENT_CHARACTER, 1),
    }
}

// PORT: Go `unicode.ToUpper` uses the simple mapping. A multi-rune full
// uppercase has no simple mapping, so the rune stays the same.

// PORT: simple case fold key, the same approach as tspath's private helper.
// Two runes are in the same Go `unicode.SimpleFold` orbit when their keys
// are equal. U+0131 (dotless i) has only a Turkic fold entry, so its orbit
// is itself, like Go. Rust and Go can use different Unicode versions; this
// matters only for runes added between those versions.

// PORT: Go `strings.EqualFold` on byte strings. Runes are decoded like Go
// (see `decode_rune`). Each rune pair must be equal, ASCII case equal, or
// in the same simple fold orbit (see `simple_fold_key`).

// Go: vfs/vfsmatch/vfsmatch.go:512 globMatcher
/// Combines include and exclude patterns for file matching.
struct GlobMatcher {
    includes: Vec<GlobPattern>,
    excludes: Vec<GlobPattern>,
    /// true if include specs were provided (even if none compiled)
    had_includes: bool,
}

// Go: vfs/vfsmatch/vfsmatch.go:518 newGlobMatcher

impl GlobMatcher {
    // Go: vfs/vfsmatch/vfsmatch.go:541 (*globMatcher).matchesFileParts




    // Go: vfs/vfsmatch/vfsmatch.go:562 (*globMatcher).matchesDirectoryParts
}

// PERF (cfgwalk1): the prefix states of the patterns of a `GlobMatcher`.
struct MatcherPrefixStates {
    excludes: Vec<Option<PrefixStates>>,
    includes: Vec<Option<PrefixStates>>,
}

// Go: vfs/vfsmatch/vfsmatch.go:579 globVisitor


// Go: vfs/vfsmatch/vfsmatch.go:647 matchFiles

// Go: vfs/vfsmatch/vfsmatch.go:676 SpecMatcher
/// Wraps multiple glob patterns for matching paths.
#[derive(Clone, Debug)]
pub struct SpecMatcher {
    patterns: Vec<GlobPattern>,
}

impl SpecMatcher {
    // Go: vfs/vfsmatch/vfsmatch.go:682 (*SpecMatcher).MatchString

    // Go: vfs/vfsmatch/vfsmatch.go:692 (*SpecMatcher).MatchIndex
}

// Go: vfs/vfsmatch/vfsmatch.go:702 NewSpecMatcher

#[cfg(test)]
mod tests {
    use super::*;

    // cfgwalk1: the prefix states (`GlobMatcher::matches_file_in` and
    // `matches_directory_in`) give the results of `match_path_parts` for
    // random patterns and paths.
    #[test]
    fn prefix_states_match_path_parts() {
        const SPECS: &[&str] = &[
            "**/*",
            "src",
            "src/**/*.ts",
            "**/node_modules",
            "*.ts",
            "a/*/b",
            "**/.git",
            "a?c/**",
            "node_modules",
            "**",
            "src/**",
            "**/*.min.js",
            "./x/../y/*",
            "/abs/path/*",
            "A/B",
            ".hidden/*",
            "**/a*b*c",
            "x/**/y/**/z",
            "\u{65e5}\u{672c}/*",
            "a/**/**/b/*",
            "*/*",
            "**/*.MIN.*",
            "../up/**/*",
            "/r",
            "/",
        ];
        const NAMES: &[&str] = &[
            "src",
            "a",
            "b",
            "node_modules",
            ".git",
            "A",
            "x",
            "y",
            "z",
            "abc",
            "aXbYc",
            "\u{65e5}\u{672c}",
            "bower_components",
            "lib.min.js",
            "foo.ts",
            "FOO.TS",
            ".hidden",
            "up",
            "path",
            "abs",
            "r",
        ];
        let mut seed: u64 = 0x9e37_79b9_7f4a_7c15;
        let mut next = |n: usize| -> usize {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            (seed % n as u64) as usize
        };
        let mut checked = 0;
        for round in 0..400 {
            let case_sensitive = round % 2 == 0;
            let pick = |k: usize, next: &mut dyn FnMut(usize) -> usize| -> Vec<String> {
                (0..k)
                    .map(|_| SPECS[next(SPECS.len())].to_string())
                    .collect()
            };
            let ni = next(3);
            let includes = pick(ni, &mut next);
            let ne = next(4);
            let excludes = pick(ne, &mut next);
            let base = if round % 5 == 0 { "/" } else { "/r" };
            let files = new_glob_matcher(&includes, &excludes, base, case_sensitive, Usage::Files);
            let dirs = new_glob_matcher(
                &includes,
                &excludes,
                base,
                case_sensitive,
                Usage::Directories,
            );
            for _ in 0..60 {
                let mut prefix = String::from(if next(6) == 0 { "/" } else { "/r/" });
                for _ in 0..next(6) {
                    prefix.push_str(NAMES[next(NAMES.len())]);
                    prefix.push('/');
                }
                let file_states = files.prefix_states(&prefix);
                let dir_states = dirs.prefix_states(&prefix);
                for name in NAMES.iter().copied().chain(["", "q.ts", "Q.d.ts"]) {
                    assert_eq!(
                        files.matches_file_in(&file_states, &prefix, name),
                        files.matches_file_parts(&prefix, name),
                        "file {prefix}{name} includes {includes:?} excludes {excludes:?} \
                         case {case_sensitive}"
                    );
                    assert_eq!(
                        dirs.matches_directory_in(&dir_states, &prefix, name),
                        dirs.matches_directory_parts(&prefix, name),
                        "dir {prefix}{name} includes {includes:?} excludes {excludes:?} \
                         case {case_sensitive}"
                    );
                    checked += 1;
                }
            }
        }
        assert_eq!(checked, 400 * 60 * (NAMES.len() + 3));
    }
}
