use crate::contentmapper::Mapper;
use crate::frontend::prelude::*;
use std::cell::OnceCell;
use std::rc::Weak;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

// This file ports tsoptions/parsedcommandline.go. `ParsedOptions` is in
// `parsed_options.rs` (tsgo#4712 moves it from core to tsoptions).
// PORT: Go `sync.Once` plus a cached field is a single-threaded `OnceCell`.
// PORT: Go methods that check `p == nil` take `&self`, which is never nil.
// A Go nil `*ParsedCommandLine` is `Option<ParsedCommandLine>` at the caller.
// PORT: Go `iter.Seq` results are eager `Vec`s. Every Go caller reads the
// whole sequence.

// Go: tsoptions/parsedcommandline.go:22 fileGlobPattern
const FILE_GLOB_PATTERN: &str = "*.{js,jsx,mjs,cjs,ts,tsx,mts,cts,json}";
// Go: tsoptions/parsedcommandline.go:23 recursiveFileGlobPattern
const RECURSIVE_FILE_GLOB_PATTERN: &str = "**/*.{js,jsx,mjs,cjs,ts,tsx,mts,cts,json}";


// ---------------------------------------------------------------------------
// Go: glob/glob.go
// PORT: `internal/glob` has one compiler caller,
// `ParsedCommandLine.WildcardDirectoryGlobs`, so it is ported in this file.
// Package-level names get a `glob_` prefix (like `json_marshal`) so the glob
// export stays unambiguous. `Parse` returns the Go error text as `Err`.
// ---------------------------------------------------------------------------

// Go: glob/glob.go:41 Glob
/// A Glob is an LSP-compliant glob pattern, as defined by the spec:
/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#documentFilter
///
/// Glob patterns can have the following syntax:
///   - `*` to match one or more characters in a path segment
///   - `?` to match on one character in a path segment
///   - `**` to match any number of path segments, including none
///   - `{}` to group sub patterns into an OR expression. (e.g. `**/*.{ts,js}`
///     matches all TypeScript and JavaScript files)
///   - `[]` to declare a range of characters to match in a path segment
///     (e.g., `example.[0-9]` to match on `example.0`, `example.1`, …)
///   - `[!...]` to negate a range of characters to match in a path segment
///     (e.g., `example.[!0-9]` to match on `example.a`, `example.b`, but
///     not `example.0`)
///
/// Expanding on this:
///   - '/' matches one or more literal slashes.
///   - any other character matches itself literally.
#[derive(Clone, Debug, Default)]
pub struct Glob {
    elems: Vec<GlobElement>, // pattern elements
}

// Go: glob/glob.go:181 element and :184 element types
/// element holds a glob pattern element.
// PORT: the Go `element` interface and its seven types are one enum.
#[derive(Clone, Debug)]
pub enum GlobElement {
    /// One or more '/' separators
    Slash,
    /// string literal, not containing /, *, ?, {}, or []
    Literal(String),
    /// *
    Star,
    /// ?
    AnyChar,
    /// **
    StarStar,
    /// {foo, bar, ...} grouping
    Group(Vec<Glob>),
    /// [a-z] character range
    CharRange { negate: bool, low: char, high: char },
}

// Go: glob/glob.go:47 Parse

// Go: glob/glob.go:52 parse
// PORT: Go indexes `pattern[0]` as a byte. Every byte it tests is ASCII, so
// the `&str` slices below stay on character boundaries.

// Go: glob/glob.go:137 readRangeRune
/// helper for decoding a rune in range elements, e.g. [a-z]
// PORT: a Rust `&str` is valid UTF-8, so Go's `errInvalidUTF8` case (a
// one-byte `RuneError`) cannot happen. An encoded U+FFFD is a valid rune in
// both.
fn glob_read_range_rune(input: &str) -> Result<(char, usize), String> {
    match input.chars().next() {
        Some(r) => Ok((r, r.len_utf8())),
        None => Err(GLOB_ERR_BAD_RANGE.to_string()),
    }
}

// Go: glob/glob.go:152 errBadRange
const GLOB_ERR_BAD_RANGE: &str = "'[' patterns must be of the form [x-y]";

impl Glob {
    // Go: glob/glob.go:157 (*Glob).parseLiteral

    // Go: glob/glob.go:215 (*Glob).Match
}

// Go: glob/glob.go:172 (*Glob).String
// PORT: Go `fmt.Stringer` is `Display`.

// Go: glob/glob.go:197-213 element String methods
// PORT: like Go, a negated range prints without the `!`.

// Go: glob/glob.go:219 match
// PORT: Go slices `[]element`; a group branch appends the rest of the
// pattern to a member's elements. Elements are borrowed here so a branch is a
// list of references. Go ignores `charRange.negate` when matching, and so
// does this port. Like Go, a `/` element that consumes the rest of the input
// indexes past the end and panics.

// Go `utf8.DecodeRuneInString` for a non-empty input: an invalid encoding
// gives `(RuneError, 1)`.
// PORT: Go `match` can split a rune (`?` skips one byte), so the input can
// start mid-rune. Rust and Go accept the same UTF-8 encodings.
fn glob_decode_rune(input: &[u8]) -> (char, usize) {
    let head = &input[..input.len().min(4)];
    let valid = match std::str::from_utf8(head) {
        Ok(s) => s,
        Err(e) => std::str::from_utf8(&head[..e.valid_up_to()]).unwrap_or_default(),
    };
    match valid.chars().next() {
        Some(c) => (c, c.len_utf8()),
        None => (char::REPLACEMENT_CHARACTER, 1),
    }
}

// Go: glob/glob.go:337 split
/// split returns the portion before and after the first slash
/// (or sequence of consecutive slashes). If there is no slash
/// it returns (input, nil).
fn glob_split(input: &[u8]) -> (&[u8], &[u8]) {
    let Some(i) = input.iter().position(|&c| c == b'/') else {
        return (input, &[]);
    };
    let first = &input[..i];
    for j in i..input.len() {
        if input[j] != b'/' {
            return (first, &input[j..]);
        }
    }
    (first, &[])
}

// PORT: the two Go maps that `ParseInputOutputNames` fills in one `Once`.

// Go: tsoptions/parsedcommandline.go:45 ParsedCommandLine
// PORT: all fields are `pub`, because Go code in the same package (the
// tsconfig parser) writes the unexported fields.
// PORT: Go `ConfigFile *TsConfigSourceFile` is `Option<Rc<...>>`, because
// `ReloadFileNamesOfParsedCommandLine` shares it with the new value.
// PORT: Go `locale` and `localeOnce` are one `OnceCell` (`locale`).

// Go: tsoptions/parsedcommandline.go:77 NewParsedCommandLine
// PORT: Go returns a new pointer; this returns the value. A nil
// `projectReferences` slice is `None`.


// Go: tsoptions/parsedcommandline.go:109 SourceOutputAndProjectReference
// PORT: Go `Resolved *ParsedCommandLine` points back at the command line
// that owns the map. An `Rc` would make a reference cycle, so this is a
// `Weak`. `ParsedCommandLine::parse_input_output_names` takes `&Rc<Self>`.

// Go: tsoptions/parsedcommandline.go:80 interface assertions
// PORT: `module.ResolvedProjectReference` and `outputpaths.OutputPathsHost`
// are satisfied by the methods below; there is no Rust trait to assert.


// Go: `*ParsedCommandLine` implements `module.ResolvedProjectReference`.

// Go: `*ParsedCommandLine` implements `outputpaths.OutputPathsHost`.

#[cfg(test)]
mod glob_tests {
    use super::*;

    // Expected values come from the pinned Go `internal/glob`.
    #[test]
    fn wildcard_directory_patterns_match_like_go() {
        let flat = glob_parse(&format!("/a/b/{FILE_GLOB_PATTERN}")).unwrap();
        let deep = glob_parse(&format!("/a/b/{RECURSIVE_FILE_GLOB_PATTERN}")).unwrap();
        assert_eq!(
            flat.to_string(),
            "/a/b/*.{js,jsx,mjs,cjs,ts,tsx,mts,cts,json}"
        );
        let cases = [
            ("/a/b/c.ts", true, true),
            ("/a/b/c/d.ts", false, true),
            ("/a/b/c.d.ts", true, true),
            ("/a/b/c.tsx", true, true),
            ("/a/b/c.css", false, false),
            ("/a//b/c.ts", true, true),
            ("/a/b/.ts", true, true),
            ("/a/b/c/d/e.json", false, true),
        ];
        for (input, in_flat, in_deep) in cases {
            assert_eq!(flat.match_(input), in_flat, "{input}");
            assert_eq!(deep.match_(input), in_deep, "{input}");
        }
    }

    #[test]
    fn parse_errors_and_elements_match_go() {
        assert_eq!(
            glob_parse("a**").unwrap_err(),
            "** may only be adjacent to '/'"
        );
        assert_eq!(glob_parse("{a").unwrap_err(), "unmatched '{'");
        assert_eq!(glob_parse("[a").unwrap_err(), GLOB_ERR_BAD_RANGE);
        let m = |pattern: &str, input: &str| glob_parse(pattern).unwrap().match_(input);
        assert!(m("a/[0-9]x", "a/5x"));
        assert!(!m("a/[0-9]x", "a/xx"));
        assert!(m("a/?", "a/b"));
        assert!(m("{a,b/c}d", "ad"));
        assert!(m("{a,b/c}d", "b/cd"));
        assert!(m("**/x", "x"));
        assert!(m("**/x", "q/r/x"));
        assert!(m("a/**", "a/b/c"));
        assert!(!m("*.ts", "a/b"));
        // Go ignores the negation when matching.
        assert!(m("[!a-c]", "b"));
        assert!(!m("[!a-c]", "é"));
    }
}

#[cfg(test)]
mod common_source_directory_tests {
    use super::*;

    fn command_line(options: CompilerOptions) -> ParsedCommandLine {
        new_parsed_command_line(
            Rc::new(options),
            vec!["/p/src/a.ts".to_string(), "/p/other/b.ts".to_string()],
            None,
            ComparePathsOptions {
                use_case_sensitive_file_names: true,
                current_directory: "/p".to_string(),
            },
        )
    }

    fn codes(command_line: &ParsedCommandLine) -> Vec<i32> {
        command_line
            .get_config_file_parsing_diagnostics()
            .iter()
            .map(Diagnostic::code)
            .collect()
    }

    // Go: tsoptions/parsedcommandline.go:176 checkSourceFilesBelongToPath
    // appends TS6059 to `p.Errors`, which GetConfigFileParsingDiagnostics
    // (:393) and WithFileNames (:93) read. GetOutputFileNames (:211) is
    // lazy: only a name under outDir or declarationDir reads
    // CommonSourceDirectory (outputpaths.go:166).
    #[test]
    fn common_source_directory_errors_are_config_errors_like_go() {
        let out_dir = command_line(CompilerOptions {
            root_dir: "/p/src".to_string(),
            out_dir: "/p/out".to_string(),
            ..Default::default()
        });
        assert!(codes(&out_dir).is_empty());
        assert_eq!(
            out_dir.get_output_file_names().next().as_deref(),
            Some("/p/out/a.js")
        );
        assert_eq!(codes(&out_dir), [6059]);
        assert_eq!(codes(&out_dir.with_file_names(Vec::new())), [6059]);

        // The first name (a.js next to a.ts) does not read the directory.
        let declaration_dir = command_line(CompilerOptions {
            root_dir: "/p/src".to_string(),
            declaration: Tristate::True,
            declaration_dir: "/p/types".to_string(),
            ..Default::default()
        });
        assert_eq!(
            declaration_dir.get_output_file_names().next().as_deref(),
            Some("/p/src/a.js")
        );
        assert!(codes(&declaration_dir).is_empty());
        assert_eq!(
            declaration_dir.get_output_file_names().collect::<Vec<_>>(),
            [
                "/p/src/a.js",
                "/p/types/a.d.ts",
                "/p/other/b.js",
                "/p/other/b.d.ts"
            ]
        );
        assert_eq!(codes(&declaration_dir), [6059]);
    }
}
