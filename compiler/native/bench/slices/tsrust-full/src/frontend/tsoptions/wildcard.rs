use crate::frontend::prelude::*;

// Go: tsoptions/wildcarddirectories.go:10 getWildcardDirectories
// PORT: Go returns a nil map for an empty include list. Every reader treats
// nil and empty the same, so this returns an empty map. Go deletes subpaths
// while it ranges over the map. The delete test depends only on the path and
// `recursive_keys`, so `retain` gives the same set.

// Go: tsoptions/wildcarddirectories.go:85 toCanonicalKey
// PORT: Go `strings.ToLower` maps each rune with the one-rune
// `unicode.ToLower` (the first char of the Rust mapping) and each byte that
// is not valid UTF-8 to U+FFFD (`go_map_runes`).

/// Go `wildcardDirectoryMatch`: the result of a wildcard directory match.
// Go: tsoptions/wildcarddirectories.go:93 wildcardDirectoryMatch
#[derive(Clone, Debug)]
struct WildcardDirectoryMatch {
    key: String,
    path: String,
    recursive: bool,
}

// Go: tsoptions/wildcarddirectories.go:99 getWildcardDirectoryFromSpec
// PORT: Go `strings.ToLower` and Rust `to_lowercase` agree on the ASCII and
// common Unicode cases that appear in paths.
