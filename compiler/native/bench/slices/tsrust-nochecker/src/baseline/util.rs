//! Port of the parts of `internal/testutil/tsbaseline/util.go` that the
//! type and symbol baselines use.


// Go: tsbaseline/util.go:24 testPathPrefixReplacer
const TEST_PATH_PREFIX_REPLACEMENTS: [(&str, &str); 7] = [
    ("/.ts/", ""),
    ("/.lib/", ""),
    ("/.src/", ""),
    ("bundled:///libs/", ""),
    ("file:///./ts/", "file:///"),
    ("file:///./lib/", "file:///"),
    ("file:///./src/", "file:///"),
];

// Go: tsbaseline/util.go:33 testPathTrailingReplacerTrailingSeparator
const TEST_PATH_TRAILING_REPLACEMENTS: [(&str, &str); 7] = [
    ("/.ts/", "/"),
    ("/.lib/", "/"),
    ("/.src/", "/"),
    ("bundled:///libs/", "/"),
    ("file:///./ts/", "file:///"),
    ("file:///./lib/", "file:///"),
    ("file:///./src/", "file:///"),
];


// Go: tsbaseline/util.go:44 removeTestPathPrefixes

// Go: tsbaseline/util.go:51 isDefaultLibraryFile

/// Go `lineDelimiter.ReplaceAllString(s, "")` with `lineDelimiter` =
/// `\r?\n` (tsbaseline/util.go:11). Removes `\r\n` and `\n`; a lone `\r`
/// stays.
pub fn remove_line_delimiters(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut prev = '\0';
    for ch in text.chars() {
        if ch == '\n' {
            if prev == '\r' {
                out.pop();
            }
        } else {
            out.push(ch);
        }
        prev = ch;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replacer_and_line_delimiters() {
        assert_eq!(
            remove_test_path_prefixes("a/.ts/b bundled:///libs/lib.d.ts", false),
            "ab lib.d.ts"
        );
        assert_eq!(
            remove_test_path_prefixes("file:///./src/x", true),
            "file:///x"
        );
        assert_eq!(remove_line_delimiters("a\r\nb\nc\rd\r\r\n"), "abc\rd\r");
        assert!(is_default_library_file("/x/lib.es5.d.ts"));
        assert!(!is_default_library_file("/x/lib.es5.ts"));
    }
}
