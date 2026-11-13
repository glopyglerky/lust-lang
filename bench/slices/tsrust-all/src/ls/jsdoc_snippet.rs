use crate::ls::prelude::*;

// Port of Go `internal/ls/jsdoc_snippet.go` (tsgo#4505): the `/** */`
// completion and its doc comment template.
//
// PORT: Go decodes runes with `stringutil.DecodeJSStringRune` and tests them
// with `stringutil.IsWhiteSpace*`. A Go rune can be a lone surrogate, which
// is not a Rust `char`; it is not white space, so `char::from_u32` failing
// counts as "not white space".
//
// PORT: `position` can cut a char: a UTF-8 client column inside a char, or
// an API UTF-16 offset inside a surrogate pair (Go `UTF16ToUTF8` maps it to
// a byte inside the char). Go slices the bytes there. `go_text_slice` gives
// each cut byte as an invalid byte unit, which is not white space, '*' or
// '/', as Go's RuneError byte is not. So the snippet checks answer as Go: a
// prefix that ends in a cut char is not a snippet prefix.

// Go: ls/jsdoc_snippet.go:19 docCommentTemplate
struct DocCommentTemplate {
    new_text: String,
}

// Go: ls/jsdoc_snippet.go:23 commentOwnerInfo
struct CommentOwnerInfo {
    comment_owner: Node,
    parameters: Vec<Node>,
    has_return: bool,
}


// Go: ls/jsdoc_snippet.go:74 isPotentiallyValidJSDocSnippetCompletionPosition

// Go: ls/jsdoc_snippet.go:124 getDocCommentTemplateAtPosition
// PORT: Go reads the reparse like any parsed file, with lazy JSDoc. Here a
// lazy JSDoc read of a published file asks the current program for the
// parser input, and the language service program has none for a file that
// is in no program ("not a Go frontend program file"). So the reparse is
// read before it is published: its node and JSDoc reads use its store (as
// for a parse cache file or in the astnav tests). It is published after the
// reads, so no unpublished store stays on this thread. `cwd` is the current
// directory of the language service program, which the publish needs.

// Go: ast.IsSourceFileJS(sourceFile) in getDocCommentTemplateAtPosition
// PORT: the reparse is read before it is published (see
// `get_doc_comment_template_at_position`), and a file that is not published
// has no `SourceFileInfo`. Its recorded parse has the script kind.

// Go: ls/jsdoc_snippet.go:181 getDocCommentEndAtPosition

// Go: ls/jsdoc_snippet.go:194 skipWhitespace

// Go: ls/jsdoc_snippet.go:208 getCommentOwnerInfo

// Go: ls/jsdoc_snippet.go:218 getCommentOwnerInfoWorker

// Go: ls/jsdoc_snippet.go:270 hasReturn

// Go: ls/jsdoc_snippet.go:287 getRightHandSideOfAssignment

// Go: ls/jsdoc_snippet.go:304 parameterDocComments

// Go: ls/jsdoc_snippet.go:328 returnsDocComment
fn returns_doc_comment(indentation: &str, new_line: &str) -> String {
    format!("{indentation} * @returns{new_line}")
}

// Go: ls/jsdoc_snippet.go:332 getIndentationStringAtPosition

// Go: ls/jsdoc_snippet.go:349 isNonEmptyJSDoc

// Go: ls/jsdoc_snippet.go:357 hasJSDocTags

// Go: ls/jsdoc_snippet.go:366 templateToSnippet

// Go: ls/jsdoc_snippet.go:377 stripJSDocTemplateIndentation

// Go: ls/jsdoc_snippet.go:390 transformJSDocTemplateLines

// Go: ls/jsdoc_snippet.go:408 lineHasOnlyJSDocAsterisk

// Go: ls/jsdoc_snippet.go:413 transformJSDocParamLine

// Go: ls/jsdoc_snippet.go:460 transformJSDocReturnsLine
fn transform_js_doc_returns_line(line: &str, snippet_index: &mut i32) -> Option<String> {
    let mut prefix = "";
    let mut rest = line;
    if rest.starts_with(' ') {
        prefix = " ";
        rest = &rest[1..];
    }
    if !rest.starts_with("* @returns") || !is_only_spaces_or_tabs(&rest["* @returns".len()..]) {
        return None;
    }
    let text = format!("{prefix}* @returns ${{{}}}", *snippet_index);
    *snippet_index += 1;
    Some(text)
}

// Go: ls/jsdoc_snippet.go:475 scanNonWhitespace

// Go: ls/jsdoc_snippet.go:492 isJSDocSnippetPrefix

// Go: ls/jsdoc_snippet.go:512 getJSDocSnippetPrefixStart
// PORT: Go returns `(int, bool)`; `None` is `false`.

// Go: ls/jsdoc_snippet.go:525 isJSDocSnippetSuffix

// Go: ls/jsdoc_snippet.go:541 getJSDocSnippetSuffixEnd
// PORT: Go returns `(int, bool)`; `None` is `(0, false)`.

// Go: ls/jsdoc_snippet.go:552 trimRightSingleLineWhitespace

// Go: ls/jsdoc_snippet.go:567 skipSingleLineWhitespace

// Go: ls/jsdoc_snippet.go:578 isOnlySingleLineWhitespace

// Go: ls/jsdoc_snippet.go:582 startsWithSingleLineWhitespace

// Go: ls/jsdoc_snippet.go:590 isOnlySpacesOrTabs
fn is_only_spaces_or_tabs(text: &str) -> bool {
    text.bytes().all(|b| b == b' ' || b == b'\t')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::parser::{self, SourceFileParseOptions};
    use std::fmt::Write;

    /// The JSDoc snippet checks answer as Go when the position cuts a char,
    /// where Go slices the line's bytes. The expected lines are from a Go
    /// test at pin N with Go 1.27.1 (`utf8cmp1/tools/gomodel` in the lane
    /// dir, an overlay test file in package `ls`). It runs the pin N helpers
    /// at each byte position of each one-line file: `isJSDocSnippetPrefix &&
    /// isJSDocSnippetSuffix` (Go `isPotentiallyValidJSDocSnippetCompletion
    /// Position`), `getJSDocSnippetPrefixStart`, `getJSDocSnippetSuffixEnd`,
    /// `getDocCommentEndAtPosition` and `skipWhitespace`. The cut chars are
    /// U+1F600, U+3000 (white space) and U+00A0 (white space).
    #[test]
    fn js_doc_snippet_checks_cut_chars_as_go() {
        let go = "\
J0 0 false - 1 0,false,false 0\n\
J0 1 false 0 - 0,false,false 1\n\
J0 2 false 0 - 0,false,false 2\n\
J0 3 false 0 - 3,true,false 4\n\
J0 4 false 0 - 4,true,false 4\n\
J0 5 false - - 0,false,false 5\n\
J0 6 false - - 0,false,false 6\n\
J0 7 false - - 0,false,false 7\n\
J0 8 false - - 0,false,false 8\n\
J1 0 false - 1 0,false,false 0\n\
J1 1 false 0 - 0,false,false 1\n\
J1 2 false 0 - 0,false,false 2\n\
J1 3 true 0 5 8,true,true 6\n\
J1 4 false - - 0,false,false 4\n\
J1 5 false - - 0,false,false 5\n\
J1 6 true 0 2 8,true,true 6\n\
J1 7 false - 1 0,false,false 7\n\
J1 8 false 7 - 0,false,false 8\n\
J2 0 false - 3 0,false,false 2\n\
J2 1 false - - 0,false,false 1\n\
J2 2 false - 1 0,false,false 2\n\
J2 3 false 2 - 0,false,false 3\n\
J2 4 false 2 - 0,false,false 4\n\
J2 5 true 2 - 5,true,false 7\n\
J2 6 false - - 0,false,false 6\n\
J2 7 true 2 - 7,true,false 7\n\
";
        let lines = ["/** \u{1F600}", "/**\u{3000}*/", "\u{A0}/**\u{A0}"];
        let opt = |n: Option<usize>| n.map_or("-".to_string(), |n| n.to_string());
        let mut port = String::new();
        for (i, line) in lines.iter().enumerate() {
            let file = Rc::new(parser::parse_source_file(
                &SourceFileParseOptions {
                    file_name: "/a.ts".to_string(),
                    path: tspath::Path("/a.ts".to_string()),
                    ..Default::default()
                },
                *line,
                ScriptKind::TS,
            ));
            crate::program::note_parsed_source_file(&file);
            let len = line.len() as i32;
            for p in 0..=len {
                let valid = is_potentially_valid_js_doc_snippet_completion_position(file.root, p);
                let prefix_start = get_js_doc_snippet_prefix_start(&go_text_slice(line, 0, p));
                let suffix_end = get_js_doc_snippet_suffix_end(&go_text_slice(line, p, len));
                let (end, ok, closing) = get_doc_comment_end_at_position(file.root, p);
                let skip = skip_whitespace(line, p);
                writeln!(
                    port,
                    "J{i} {p} {valid} {} {} {end},{ok},{closing} {skip}",
                    opt(prefix_start),
                    opt(suffix_end)
                )
                .unwrap();
            }
        }
        assert_eq!(port, go);
    }
}
