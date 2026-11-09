use crate::ls::prelude::*;

// Port of Go `ls/string_completions.go`.
//
// PORT (whole file):
// - Go `*compiler.Program` is `&compiler::NewProgram` (plan
//   D-PROGRAM). Package.json lookups and the typings cache location are
//   `NewProgram` methods, run on the dispatch thread.
// - Go passes `program` where a `modulespecifiers.ModuleSpecifierGenerationHost`
//   is needed. The host for the installed program is
//   `modulespecifiers::ProgramHost` (installed program, plan D-LS1).
// - Go `*ast.SourceFile` passed as an `ast.HasFileName` is
//   `new_has_file_name(file name, path)` (Go `ast.NewHasFileName`).
// - Go `file.Path()` is `source_file_info(file).path` (a `String`).
// - `*checker.StringLiteralType` is `TypeId`. Go `t.AsLiteralType().Value().(string)`
//   panics when the value is not a string; so does the Rust `let ... else`.
// - Go `moduleCompletionNameAndKindSet.names` is a Go map, and the results are
//   collected with `slices.Collect(maps.Values(...))`. It is an `IndexMap`
//   in insertion order. PORT: Go map order is random; the oracle compares
//   that output without order.
// - Go `fromPaths *pathCompletions` (tsgo#4712) is `Option<PathCompletions>`.
//   It is set even when it has no entries.
// - Go byte-offset string cuts whose boundary is not guaranteed by a prefix
//   check use `String::from_utf8_lossy` on the bytes (Go keeps raw bytes;
//   Rust text must stay valid UTF-8).

use crate::astnav;
use crate::flags_macros::go_enum;
use crate::frontend::stringutil_ls::{COMPARISON_GREATER_THAN, COMPARISON_LESS_THAN, Comparison};
use crate::frontend::{compiler, module, packagejson, tsoptions, tspath};
use crate::ls::lsutil;
use crate::lsp::lsproto;
use crate::modulespecifiers;
use crate::spanmap::Fidelity;

// Go: ls/string_completions.go:30 completionsFromTypes

// Go: ls/string_completions.go:35 completionsFromProperties

// Go: ls/string_completions.go:40 pathCompletion
#[derive(Clone, Debug)]
struct PathCompletion {
    name: String,
    kind: lsutil::ScriptElementKind,
    extension: String,
}

// Go: ls/string_completions.go:46 pathCompletions
// PORT: Go `replacementSpan *lsproto.Range`; nil is `None`.
#[derive(Clone, Debug, Default)]
struct PathCompletions {
    entries: Vec<PathCompletion>,
    replacement_span: Option<lsproto::Range>,
}

// Go: ls/string_completions.go:51 stringLiteralCompletions
// PORT: Go `*T` fields are `Option<T>`.


// Go: ls/string_completions.go:440 fromContextualType

// Go: ls/string_completions.go:446 toCompletionsFromTypes

// Go: ls/string_completions.go:456 toStringLiteralCompletionsFromTypes

// Go: ls/string_completions.go:466 fromUnionableLiteralType

// Go: ls/string_completions.go:555 stringLiteralCompletionsForObjectLiteral

// Go: ls/string_completions.go:578 stringLiteralCompletionsFromProperties



// Go: ls/string_completions.go:612 toPathCompletions


// Go: ls/string_completions.go:633 moduletToScriptElementKind

// Go: ls/string_completions.go:645 isAnyDirectorySeparator
fn is_any_directory_separator(r: char) -> bool {
    r == '/' || r == '\\'
}

// Go: ls/string_completions.go:650 getDirectoryFragmentRange
// Replace everything after the last directory separator that appears
// PORT: Go works on the bytes of `text`. `text` is the Go bytes
// (`go_string_bytes`), and `text_start` and the range are Go byte offsets.
// A separator is ASCII, so the last separator byte is Go's last separator
// rune.
fn get_directory_fragment_range(text: &[u8], text_start: i32) -> Option<TextRange> {
    let index = text
        .iter()
        .rposition(|&b| is_any_directory_separator(char::from(b)));
    let mut offset = 0;
    if let Some(index) = index {
        offset = index + 1;
    }
    let length = text.len() - offset;
    if length == 0 {
        return None;
    }
    Some(TextRange::new(
        text_start + offset as i32,
        text_start + offset as i32 + length as i32,
    ))
}


// Go: ls/string_completions.go:875 getFragmentDirectory

// Go: ls/string_completions.go:885 getPatternFromFirstMatchingCondition

// Go: ls/string_completions.go:904 getAmbientModuleCompletions

// Go: ls/string_completions.go:923 getAmbientModuleName
// PORT: reading the symbol takes the symbol arena (as for Go `ast`
// functions that take a symbol).


// Go: ls/string_completions.go:1000 tryRemoveDirectoryPrefix
// tsgo#4900: `TrimFilePathPrefix` cuts the prefix by runes, not by its byte
// length.


// Go: ls/string_completions.go:1050 getSupportedExtensionsForModuleResolution

// Go: ls/string_completions.go:1074 moduleResolutionUsesNodeModules
fn module_resolution_uses_node_modules(module_resolution: ModuleResolutionKind) -> bool {
    module_resolution >= ModuleResolutionKind::NODE16
        && module_resolution <= ModuleResolutionKind::NODE_NEXT
        || module_resolution == ModuleResolutionKind::BUNDLER
}

// Go: ls/string_completions.go:1080 isPathRelativeToScript
// Returns true if the path is explicitly relative (i.e. relative to . or ..)
fn is_path_relative_to_script(path: &str) -> bool {
    path.starts_with("./") || path.starts_with("../")
}


// Go: ls/string_completions.go:1155 getBaseDirectoriesFromRootDirs
// getBaseDirectoriesFromRootDirs takes a script path and returns paths for all potential folders
// that could be merged with its containing folder via the "rootDirs" compiler option.

// Go: ls/string_completions.go:1195 deduplicateStrings
fn deduplicate_strings(slice: Vec<String>) -> Vec<String> {
    if slice.len() <= 1 {
        return slice;
    }
    let mut seen: FxHashSet<String> = FxHashSet::default();
    let mut result = Vec::new();
    for s in slice {
        if !seen.contains(&s) {
            seen.insert(s.clone());
            result.push(s);
        }
    }
    result
}

// Go: ls/string_completions.go:1210 deduplicateModuleCompletions

// Go: ls/string_completions.go:1231 moduleCompletionKind
go_enum!(ModuleCompletionKind, i32 {
    DIRECTORY = 0; // moduleCompletionKindDirectory
    FILE = 1; // moduleCompletionKindFile
    EXTERNAL_MODULE_NAME = 2; // moduleCompletionKindExternalModuleName
});

// Go: ls/string_completions.go:1239 moduleCompletionNameAndKind
#[derive(Clone, Debug, Default)]
struct ModuleCompletionNameAndKind {
    name: String,
    kind: ModuleCompletionKind,
    extension: String,
}

// Go: ls/string_completions.go:1245 moduleCompletionNameAndKindSet
// PORT: Go map; `IndexMap` in insertion order (see the file header).


// Go: ls/string_completions.go:1256 extensionOptions
#[derive(Clone, Debug)]
struct ExtensionOptions {
    extensions_to_search: Vec<String>,
    reference_kind: ReferenceKind,
    importing_source_file: Node,
    ending_preference: modulespecifiers::ImportModuleSpecifierEndingPreference,
    resolution_mode: ResolutionMode,
}

// Go: ls/string_completions.go:1264 referenceKind
go_enum!(ReferenceKind, i32 {
    FILE_NAME = 0; // referenceKindFileName
    MODULE_SPECIFIER = 1; // referenceKindModuleSpecifier
});


// Go: ls/string_completions.go:1598 getFileExtension


// Go: ls/string_completions.go:1822 containsSlash

// Go: ls/string_completions.go:1826 withoutStartAndEnd
fn without_start_and_end(s: &str, start: &str, end: &str) -> Option<String> {
    if s.starts_with(start) && s.ends_with(end) && s.len() >= start.len() + end.len() {
        return Some(s[start.len()..s.len() - end.len()].to_string());
    }
    None
}

// Go: ls/string_completions.go:1834 removeLeadingDirectorySeparator

// Go: ls/string_completions.go:1838 getPossibleOriginalInputPathWithoutChangingExt

// Go: ls/string_completions.go:1855 getFilenameWithExtensionOption

// Go: ls/string_completions.go:1911 walkUpParentheses

// Go: ls/string_completions.go:1922 getStringLiteralTypes
// PORT: Go `uniques *collections.Set[string]` can be nil: `Option<&mut ..>`;
// nil makes a fresh set as in Go.

// Go: ls/string_completions.go:1943 getAlreadyUsedTypesInStringLiteralUnion

// Go: ls/string_completions.go:1958 hasIndexSignature

// Go: ls/string_completions.go:1966 isRequireCallArgument
// Matches
//
//	require(""
//	require("")

// Go: ls/string_completions.go:1971 kindModifiersFromExtension
fn kind_modifiers_from_extension(extension: &str) -> lsutil::ScriptElementKindModifier {
    match extension {
        tspath::EXTENSION_DTS => lsutil::ScriptElementKindModifier::DTS,
        tspath::EXTENSION_JS => lsutil::ScriptElementKindModifier::JS,
        tspath::EXTENSION_JSON => lsutil::ScriptElementKindModifier::JSON,
        tspath::EXTENSION_JSX => lsutil::ScriptElementKindModifier::JSX,
        tspath::EXTENSION_TS => lsutil::ScriptElementKindModifier::TS,
        tspath::EXTENSION_TSX => lsutil::ScriptElementKindModifier::TSX,
        tspath::EXTENSION_DMTS => lsutil::ScriptElementKindModifier::DMTS,
        tspath::EXTENSION_MJS => lsutil::ScriptElementKindModifier::MJS,
        tspath::EXTENSION_MTS => lsutil::ScriptElementKindModifier::MTS,
        tspath::EXTENSION_DCTS => lsutil::ScriptElementKindModifier::DCTS,
        tspath::EXTENSION_CJS => lsutil::ScriptElementKindModifier::CJS,
        tspath::EXTENSION_CTS => lsutil::ScriptElementKindModifier::CTS,
        tspath::EXTENSION_TS_BUILD_INFO => crate::core::go_panic(format!(
            "Extension {} is unsupported.",
            tspath::EXTENSION_TS_BUILD_INFO
        )),
        _ => lsutil::ScriptElementKindModifier::NONE,
    }
}

// Go: ls/string_completions.go:2004 getStringLiteralCompletionsFromSignature


// Go: ls/string_completions.go:2105 isInReferenceComment

// Go: ls/string_completions.go:2114 hasTripleSlashPrefix

// Go: ls/string_completions.go:2134 parseTripleSlashDirectiveFragment
// Matches a triple slash reference directive with an incomplete string literal for its path.
// Used to determine if the caret is currently within the string literal and capture the literal
// fragment for completions.
// For example, this matches
//
// /// <reference path="fragment
//
// but not
//
// /// <reference path="fragment"

// Returns (prefix, kind, toComplete, ok) where:
//   - prefix is everything up to and including the opening quote
//   - kind is either "path" or "types"
//   - toComplete is the fragment after the opening quote
//   - ok indicates whether the match was successful


// Go: ls/string_completions_test.go (tsgo#4900)
#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    // Go: ls/string_completions_test.go:17 TestTryRemoveDirectoryPrefixCaseFoldingShrinksPrefix
    // Each Kelvin sign '\u212A' below case-folds to the single-byte 'k', so the raw
    // prefix is longer in bytes (15) than path (12), even though path's canonical
    // form is case-insensitively prefixed by prefix's canonical form.
    #[test]
    fn test_try_remove_directory_prefix_case_folding_shrinks_prefix() {
        let prefix = "/a/\u{212A}\u{212A}\u{212A}\u{212A}";
        let path = "/a/kkkk/x.ts";
        let actual =
            try_remove_directory_prefix(path, prefix, false /*useCaseSensitiveFileNames*/);
        assert_eq!(actual.as_deref(), Some("x.ts"));
    }

    /// The triple-slash fragment of `getTripleSlashReferenceCompletions`
    /// is Go's when the position cuts a char, where Go slices the comment's
    /// bytes and keeps the cut bytes in `toComplete`. The expected lines are
    /// from a Go test at pin N with Go 1.27.1 (`utf8cmp1/tools/gomodel` in
    /// the lane dir, an overlay test file in package `ls`) that runs, at
    /// each position: `parseTripleSlashDirectiveFragment(text[:p])` (ok,
    /// `len(prefix)`, kind and the `toComplete` bytes),
    /// `getDirectoryFragmentRange(toComplete, len(prefix))` and the bytes of
    /// `getFragmentDirectory(toComplete)`. Go's range always ends at the
    /// position, as the port's range does.
    #[test]
    fn triple_slash_fragment_cut_chars_as_go() {
        let go = "\
T0 20 false 0 \"\"  nil \n\
T0 21 true 21 \"path\"  nil \n\
T0 22 true 21 \"path\" 2e 21-22 \n\
T0 23 true 21 \"path\" 2e2f nil 2e2f\n\
T0 24 true 21 \"path\" 2e2fc3 23-24 2e\n\
T0 25 true 21 \"path\" 2e2fc3a9 23-25 2e\n\
T0 26 true 21 \"path\" 2e2fc3a92f nil 2e2fc3a92f\n\
T0 27 true 21 \"path\" 2e2fc3a92ff0 26-27 2e2fc3a9\n\
T0 28 true 21 \"path\" 2e2fc3a92ff09f 26-28 2e2fc3a9\n\
T0 29 true 21 \"path\" 2e2fc3a92ff09f98 26-29 2e2fc3a9\n\
T0 30 true 21 \"path\" 2e2fc3a92ff09f9880 26-30 2e2fc3a9\n\
T1 21 false 0 \"\"  nil \n\
T1 22 true 22 \"types\"  nil \n\
T1 23 true 22 \"types\" c3 22-23 \n\
T1 24 true 22 \"types\" c3a9 22-24 \n\
T2 3 false 0 \"\"  nil \n\
T2 4 false 0 \"\"  nil \n\
T2 5 false 0 \"\"  nil \n\
T2 6 false 0 \"\"  nil \n\
";
        let cases = [
            ("/// <reference path=\"./\u{E9}/\u{1F600}", 20, 30),
            ("/// <reference types=\"\u{E9}", 21, 24),
            ("///\u{3000}<reference path=\"a", 3, 6),
        ];
        let hex = |s: &str| {
            go_string_bytes(s)
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        };
        let mut port = String::new();
        for (i, (text, from, to)) in cases.into_iter().enumerate() {
            for p in from..=to {
                let slice = go_text_slice(text, 0, p);
                let (prefix, kind, to_complete, ok) = parse_triple_slash_directive_fragment(&slice);
                let range = get_directory_fragment_range(
                    &go_string_bytes(to_complete),
                    go_len(prefix) as i32,
                )
                .map_or("nil".to_string(), |range| format!("{}-{p}", range.pos()));
                writeln!(
                    port,
                    "T{i} {p} {ok} {} {kind:?} {} {range} {}",
                    prefix.len(),
                    hex(to_complete),
                    hex(&get_fragment_directory(to_complete))
                )
                .unwrap();
            }
        }
        assert_eq!(port, go);
    }

    /// The range that module name completions replace is Go's: Go adds byte
    /// offsets of the literal's value to the source offset of its text, so
    /// a value whose port form differs from the source's (an escape, a WTF-8
    /// surrogate that the value fuses into one unit) or that has a marker
    /// unit must be counted in Go bytes. The expected lines are from a Go
    /// test at pin N with Go 1.27.1 (`followups17/tools/gomodel` in the lane
    /// dir, an overlay test file in package `ls`) that prints
    /// `getDirectoryFragmentRange(node.Text(), textStart)` of
    /// `getStringLiteralCompletionsFromModuleNames` for the module specifier,
    /// in Go bytes. It numbers these texts after its 13 texts of
    /// `findallreferences_p1::tests::range_of_a_string_literal_counts_go_bytes`.
    #[test]
    fn module_name_fragment_range_counts_go_bytes() {
        use crate::frontend::parser::{SourceFileParseOptions, parse_source_file};
        use crate::frontend::tspath::Path;
        use crate::scanner_util::go_string_from_bytes;
        let go = "\
M13 22-23
M14 21-25
M15 22-23
M16 21-25
M17 21-25
M18 22-26
M19 22-26
M20 21-23
M21 20-22
M22 21-23
M23 21-22
M24 21-26
";
        let texts: [&[u8]; 12] = [
            b"import a from \"./d\xed\xa0\x80/f\";\n",
            b"import b from \"./sub/a\xed\xa0\x80\";\n",
            b"import c from \"./d\\uD800/f\";\n",
            b"import d from \"./sub/a\\uD800\";\n",
            b"import e from \"./sub/a\\uFDD0\";\n",
            b"import f from \"./d\\uFDD0/a\\uFDD0\";\n",
            b"import g from \"./d\xef\xb7\x90/a\xef\xb7\x90\";\n",
            b"import h from \"./sub/a\xff\";\n",
            b"import i from \"./d\xff/a\xff\";\n",
            b"import j from \"./sub/\\xFF\";\n",
            b"import k from \"./sub/\xff\n",
            b"import l from \"./sub/\\uD83D\\uDE00x\";\n",
        ];
        let mut port = String::new();
        for (i, bytes) in texts.into_iter().enumerate() {
            let text = go_string_from_bytes(bytes.to_vec());
            let file = parse_source_file(
                &SourceFileParseOptions {
                    file_name: "/a.ts".to_string(),
                    path: Path("/a.ts".to_string()),
                    ..Default::default()
                },
                crate::ast::FileText::new(text.clone(), false),
                ScriptKind::TS,
            );
            let specifier = file.root.statements().get(0).module_specifier();
            let range = module_name_fragment_range(file.root, specifier).map_or(
                "nil".to_string(),
                |range| {
                    format!(
                        "{}-{}",
                        go_byte_offset(&text, range.pos()),
                        go_byte_offset(&text, range.end())
                    )
                },
            );
            writeln!(port, "M{} {range}", i + 13).unwrap();
        }
        assert_eq!(port, go);
    }
}
