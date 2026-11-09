//! Port of semver/version.go and semver/version_range.go.
//!
//! PORT: Go compiles its patterns with `regexp`. This crate has no regex
//! dependency, so each pattern has a hand-written matcher below
//! (`match_version_pattern`, `is_prerelease_part`, `is_build_part`,
//! `match_hyphen`, `match_range`). Each matcher gives the same match result
//! and the same submatches as Go RE2 leftmost-first matching on the anchored
//! pattern. `(?i)` in RE2 uses Unicode simple case folding, so `[a-z]`
//! also matches `ſ` (U+017F) and the Kelvin sign (U+212A). `\d` and `\s` are
//! ASCII only in RE2 (`\s` is `[\t\n\f\r ]`).

#[allow(unused_imports)]
use crate::frontend::prelude::*;

use std::cmp::Ordering;

// `[a-z]` under `(?i)` (RE2 simple case folding).
fn is_fold_letter(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '\u{17F}' || c == '\u{212A}'
}

// `[a-z0-9-.]` under `(?i)`.
fn is_part_char(c: char) -> bool {
    is_fold_letter(c) || c.is_ascii_digit() || c == '-' || c == '.'
}

// RE2 `\s`.
fn is_re_space(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\x0C' | '\r' | ' ')
}

// Matches `0|[1-9]\d*` (or `[x*0]|[1-9]\d*` under `(?i)` when `wildcards`)
// at the start of `s`. Returns the length of the match.

// Matches a run of `[a-z0-9-.]+` (under `(?i)`) at the start of `s`.

// Go: semver/version.go:19 versionRegexp, semver/version_range.go:28 partialRegExp
// PORT: hand-written matcher for Go `versionRegexp` (`wildcards == false`)
// and `partialRegExp` (`wildcards == true`):
// `(?i)^(N)(?:\.(N)(?:\.(N)(?:-([a-z0-9-.]+))?(?:\+([a-z0-9-.]+))?)?)?$`.
// Returns the five submatches ("" when a group does not take part), like
// Go `FindStringSubmatch(text)[1:]`.

// Go: semver/version.go:28 prereleasePartRegexp
// PORT: hand-written matcher for `(?i)^(?:0|[1-9]\d*|[a-z-][a-z0-9-]*)$`.
fn is_prerelease_part(s: &str) -> bool {
    if s == "0" {
        return true;
    }
    let mut chars = s.chars();
    match chars.next() {
        Some('1'..='9') => chars.all(|c| c.is_ascii_digit()),
        Some(c) if is_fold_letter(c) || c == '-' => {
            chars.all(|c| is_fold_letter(c) || c.is_ascii_digit() || c == '-')
        }
        _ => false,
    }
}

// Go: semver/version.go:27 prereleaseRegexp
// PORT: `(?i)^P(?:\.P)*$` with P the part pattern. A dot-separated
// sequence of parts, each matching `is_prerelease_part`.
fn is_prerelease(s: &str) -> bool {
    s.split('.').all(is_prerelease_part)
}

// Go: semver/version.go:37 buildPartRegExp
// PORT: hand-written matcher for `(?i)^[a-z0-9-]+$`.
fn is_build_part(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| is_fold_letter(c) || c.is_ascii_digit() || c == '-')
}

// Go: semver/version.go:36 buildRegExp
// PORT: `(?i)^[a-z0-9-]+(?:\.[a-z0-9-]+)*$`.
fn is_build(s: &str) -> bool {
    s.split('.').all(is_build_part)
}

// Go: semver/version.go:42 numericIdentifierRegExp
// PORT: hand-written matcher for `^(?:0|[1-9]\d*)$`.

// Go: semver/version.go:44 Version
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
    prerelease: Vec<String>,
    build: Vec<String>,
}

// Go: semver/version.go:52 versionZero
fn version_zero() -> Version {
    Version {
        prerelease: vec!["0".to_string()],
        ..Version::default()
    }
}

const COMPARISON_LESS_THAN: i32 = -1;
const COMPARISON_EQUAL_TO: i32 = 0;
const COMPARISON_GREATER_THAN: i32 = 1;

fn cmp_to_i32(o: Ordering) -> i32 {
    match o {
        Ordering::Less => COMPARISON_LESS_THAN,
        Ordering::Equal => COMPARISON_EQUAL_TO,
        Ordering::Greater => COMPARISON_GREATER_THAN,
    }
}

impl Version {
    // Go: semver/version.go:56 incrementMajor
    // PORT: Go does not overflow-check `+ 1`. `wrapping_add` keeps the
    // Go uint32 wraparound.
    fn increment_major(&self) -> Version {
        Version {
            major: self.major.wrapping_add(1),
            ..Version::default()
        }
    }

    // Go: semver/version.go:62 incrementMinor
    fn increment_minor(&self) -> Version {
        Version {
            major: self.major,
            minor: self.minor.wrapping_add(1),
            ..Version::default()
        }
    }

    // Go: semver/version.go:69 incrementPatch
    fn increment_patch(&self) -> Version {
        Version {
            major: self.major,
            minor: self.minor,
            patch: self.patch.wrapping_add(1),
            ..Version::default()
        }
    }

    // Go: semver/version.go:83 Compare
    // PORT: Go takes pointers and orders nil first. Rust callers always
    // pass a version, so the nil cases are gone. The `a == b` pointer
    // shortcut gives the same result as the full compare.

    // Go: semver/version.go:190 String
}

// Go: semver/version.go:123 comparePreReleaseIdentifiers

// Go: semver/version.go:143 comparePreReleaseIdentifier

// Go: semver/version.go:202 SemverParseError
#[derive(Clone, Debug)]
pub struct SemverParseError {
    orig_input: String,
}

impl SemverParseError {
    // Go: semver/version.go:206 Error
    // PORT: Go `%q` quoting. For the inputs this sees (package.json keys
    // and the compiler version) `{:?}` gives the same text, except for
    // non-printable characters.
    pub fn error(&self) -> String {
        format!("Could not parse version string from {:?}", self.orig_input)
    }
}

// Go: semver/version.go:210 TryParseVersion
// PORT: Go returns `(Version, error)` with a partly filled version on
// error. No caller reads that version, so an error is `Err(message)`.

// Go: semver/version.go:263 MustParse
// PORT: named `must_parse_version` so the glob export does not claim the
// generic name `must_parse`.

// Go: semver/version.go:271 getUintComponent
// PORT: Go `strconv.ParseUint(text, 10, 32)`. Only ASCII digits are
// accepted; an empty string, a sign or a value above u32::MAX is an error.

// Go: semver/version_range.go:43 VersionRange
#[derive(Clone, Debug, Default)]
pub struct VersionRange {
    alternatives: Vec<Vec<VersionComparator>>,
}

// Go: semver/version_range.go:47 versionComparator
#[derive(Clone, Debug)]
struct VersionComparator {
    operator: ComparatorOperator,
    operand: Version,
}

// Go: semver/version_range.go:52 comparatorOperator
// PORT: Go uses a string type and also stores the raw `~`, `^` and ``
// operators in it while parsing. Those never reach a stored comparator,
// so the stored operator is an enum and `parse_comparator` switches on
// the raw `&str`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ComparatorOperator {
    RangeLessThan,
    RangeLessThanEqual,
    RangeEqual,
    RangeGreaterThanEqual,
    RangeGreaterThan,
}

impl ComparatorOperator {
    fn as_str(self) -> &'static str {
        match self {
            ComparatorOperator::RangeLessThan => "<",
            ComparatorOperator::RangeLessThanEqual => "<=",
            ComparatorOperator::RangeEqual => "=",
            ComparatorOperator::RangeGreaterThanEqual => ">=",
            ComparatorOperator::RangeGreaterThan => ">",
        }
    }
}

impl VersionRange {
    // Go: semver/version_range.go:62 String

    // Go: semver/version_range.go:97 Test
}

// Go: semver/version_range.go:68 formatDisjunction

// Go: semver/version_range.go:83 formatAlternative

// Go: semver/version_range.go:92 formatComparator

// Go: semver/version_range.go:101 testDisjunction

// Go: semver/version_range.go:116 testAlternative

// Go: semver/version_range.go:125 testComparator

// Go: semver/version_range.go:143 TryParseVersionRange

// PORT: Go `whitespaceRegExp.Split(r, -1)` for `\s+`. `r` is already
// trimmed, so this is a split on each maximal run of RE2 `\s`.
fn split_whitespace_re(r: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut beg = 0;
    let mut chars = r.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if is_re_space(c) {
            let mut end = i + c.len_utf8();
            while let Some(&(j, d)) = chars.peek() {
                if !is_re_space(d) {
                    break;
                }
                end = j + d.len_utf8();
                chars.next();
            }
            out.push(&r[beg..i]);
            beg = end;
        }
    }
    out.push(&r[beg..]);
    out
}

// Go: semver/version_range.go:148 parseAlternatives

// `[a-z0-9-+.*]` under `(?i)`.
fn is_range_operand_char(c: char) -> bool {
    is_fold_letter(c) || c.is_ascii_digit() || matches!(c, '-' | '+' | '.' | '*')
}



// Go: semver/version_range.go:33 hyphenRegExp
// PORT: hand-written matcher for
// `(?i)^\s*([a-z0-9-+.*]+)\s+-\s+([a-z0-9-+.*]+)\s*$`. The operand class
// has no space and each space run must be followed by a non-space, so
// the maximal runs are the only possible match.

// Go: semver/version_range.go:41 rangeRegExp
// PORT: hand-written matcher for
// `(?i)^([~^<>=]|<=|>=)?\s*([a-z0-9-+.*]+)$`. With leftmost-first
// alternation, a one-character `<` or `>` followed by `=` cannot match
// (the operand class has no `=`), so `<=` and `>=` win there.

// Go: semver/version_range.go:188 parseHyphen

// Go: semver/version_range.go:235 partialVersion
struct PartialVersion {
    version: Version,
    major_str: String,
    minor_str: String,
    patch_str: String,
}

// Go: semver/version_range.go:243 parsePartial
// Produces a "partial" version

// Go: semver/version_range.go:321 parseComparator

// Go: semver/version_range.go:436 isWildcard
fn is_wildcard(text: &str) -> bool {
    text == "*" || text == "x" || text == "X"
}
