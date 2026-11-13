//! Port of typescript-go `internal/parser/utilities.go`,
//! `internal/parser/types.go` and `internal/ast/parseoptions.go`.
//!
//! PORT: Go `parser.ParseFlags` (types.go) is already ported as
//! `crate::flags::ParseFlags` with the Go values, so types.go adds nothing
//! here.
//!
//! PORT: parseoptions.go is in package `ast`. The plan gives it to this unit
//! without a file, so it is here, next to the parser code that uses it.

use crate::frontend::prelude::*;

// ── parser/utilities.go ────────────────────────────────────────────────

// Go: parser/utilities.go:11 getLanguageVariant

// Go: parser/utilities.go:20 tokenIsIdentifierOrKeyword
// Go: parser/utilities.go:24 tokenIsIdentifierOrKeywordOrGreaterThan
// PORT: the same two functions from Go checker/utilities.go are already
// public in `checker/utilities_p1.rs` (`token_is_identifier_or_keyword`,
// `token_is_identifier_or_keyword_or_greater_than`) with the same bodies. A
// second public copy would make the prelude glob ambiguous, so the parser
// uses those.

// Go: parser/utilities.go:28 GetJSDocCommentRanges
// PORT: Go appends to the `commentRanges` slice and filters it in place
// (`slices.DeleteFunc`). Here the vector is taken by value and returned.

// Go: parser/utilities.go:50 isKeywordOrPunctuation
pub fn is_keyword_or_punctuation(token: SyntaxKind) -> bool {
    is_keyword_kind(token) || is_punctuation_kind(token)
}

// Go: parser/utilities.go:54 isJSDocLikeText
// PORT: `pub(super)`, because `printer/utilities.rs` has a crate-visible
// `is_js_doc_like_text` (the Go printer function of the same name, with other
// parameters). Parser files call this one as `super::utilities::is_js_doc_like_text`.
pub(super) fn is_js_doc_like_text(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() >= 4 && bytes[1] == b'*' && bytes[2] == b'*' && bytes[3] != b'/'
}

// ── ast/parseoptions.go ────────────────────────────────────────────────

/// Go `ast.SourceFileParseOptions`.
// Go: ast/parseoptions.go:8 SourceFileParseOptions
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SourceFileParseOptions {
    pub file_name: String,
    pub path: Path,
    pub external_module_indicator_options: ExternalModuleIndicatorOptions,
}

/// Go `ast.ExternalModuleIndicatorOptions`.
// Go: ast/parseoptions.go:14 ExternalModuleIndicatorOptions
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ExternalModuleIndicatorOptions {
    pub jsx: bool,
    pub force: bool,
}

// Go: ast/parseoptions.go:19 GetExternalModuleIndicatorOptions

// Go: ast/parseoptions.go:44 isFileForcedToBeModuleByFormatExtensions
const IS_FILE_FORCED_TO_BE_MODULE_BY_FORMAT_EXTENSIONS: [&str; 4] =
    [EXTENSION_CJS, EXTENSION_CTS, EXTENSION_MJS, EXTENSION_MTS];

// Go: ast/parseoptions.go:46 isFileForcedToBeModuleByFormat

// Go: ast/parseoptions.go:56 SetExternalModuleIndicator
// PORT: Go takes `*ast.SourceFile`. The SourceFile fields live in
// `ParsedSourceFile` (plan contract 6).

thread_local! {
    /// Set when `get_external_module_indicator` reads its options. A
    /// detached parse reports it (`parse_source_file_detached`).
    static MODULE_INDICATOR_OPTIONS_READ: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub(crate) fn reset_module_indicator_options_read() {
    MODULE_INDICATOR_OPTIONS_READ.with(|read| read.set(false));
}

pub(crate) fn module_indicator_options_read() -> bool {
    MODULE_INDICATOR_OPTIONS_READ.with(std::cell::Cell::get)
}

// Go: ast/parseoptions.go:60 getExternalModuleIndicator



// Go: ast/parseoptions.go:86 isFileProbablyExternalModule
// PORT: takes the SourceFile node.

// Go: ast/parseoptions.go:95 isAnExternalModuleIndicatorNode

// Go: ast/parseoptions.go:101 getImportMetaIfNecessary

// Go: ast/parseoptions.go:108 findChildNode

// Go: ast/parseoptions.go:122 isFileModuleFromUsingJSXTag

// Go: ast/parseoptions.go:129 walkTreeForJSXTags
// This is a somewhat unavoidable full tree walk to locate a JSX tag - `import.meta` requires the same,
// but we avoid that walk (or parts of it) if at all possible using the `PossiblyContainsImportMeta` node flag.
// Unfortunately, there's no `NodeFlag` space to do the same for JSX.
