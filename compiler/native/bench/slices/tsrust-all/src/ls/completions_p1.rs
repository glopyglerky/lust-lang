use crate::ls::prelude::*;

// Port of Go `ls/completions.go` lines 1-1708: the completion entry points,
// the shared completion types and `getCompletionData`.
//
// PORT (whole file):
// - The shared completion types follow map-ls-completions.md section 2.3.
// - Go `(T, error)` results are `Result<T, GoError>`. A Go
//   `return globalsSearchFail, err` is `Err(err)`; the caller never reads the
//   search value when the error is set.
// - `getCompletionData` has 18 closures over shared locals. They are methods
//   of the private `GetCompletionDataState`, which holds every local that the
//   closures read or change. The methods keep the closure names and Go order.
// - Checker lease: PORTING "Programs and checkers". Helpers take
//   `type_checker: &mut Checker`.

use crate::flags_macros::{go_enum, go_flags};
use crate::frontend::scanner::scanner_p1::{RUNE_ERROR, };
use crate::spanmap::Feature;
use std::sync::LazyLock;

// Go: ls/completions.go:35 ErrNeedsAutoImports


// Go: ls/completions.go:110 CompletionItem
// PORT: Go embeds `*lsproto.CompletionItem` and never stores a nil one. The
// embedded item is the field `completion_item`; `Deref`/`DerefMut` give the Go
// promoted fields (`item.label`, `item.sort_text`, ...).



// Go: ls/completions.go:115 CompletionList

// Go: ls/completions.go:122 ensureItemData
// PORT: Go fills `item.Data` through the list pointer; the list is moved in
// and returned. `data.position` is a Go byte offset in Go, and the client
// sends it back on resolve. Port offsets differ from Go offsets after a
// marker unit (see `scanner_util::GO_STRING_MARKER`), so the list leaves
// with Go offsets (`go_byte_offset`) and `resolve_completion_item` maps them
// back. The items that `create_lsp_completion_item` makes hold the port
// `pos` until here, so the text is scanned once for the list, not once for
// each item.

// Go: ls/completions.go:139 supplementalFileIndex
// PORT: Go `*int32`; nil is `None`.

// Go: ls/completions.go:152 sourceFileForSupplementalFileIndex
// PORT: Go `*ast.SourceFile` is the file root `Node` (nil is `Node::NIL`).
// Go `*int32` is `Option<i32>`.

// Go: ls/completions.go:164 completionData
// *completionDataData | *completionDataKeyword | *completionDataJSDocTagName | *completionDataJSDocTag | *completionDataJSDocParameterName
// PORT: Go `type completionData = any` is an enum over the five pointer types.
// A nil `completionData` is `Option::None` around it.

// Go: ls/completions.go:166 completionDataData
#[derive(Clone, Debug, Default)]
pub struct CompletionDataData {
    pub symbols: Vec<SymbolId>,
    pub auto_imports: Vec<autoimport::FixAndExport>,
    pub completion_kind: CompletionKind,
    pub is_in_snippet_scope: bool,
    // Note that the presence of this alone doesn't mean that we need a conversion. Only do that if the completion is not an ordinary identifier.
    pub property_access_to_convert: Node,
    pub is_new_identifier_location: bool,
    pub location: Node,
    pub keyword_filters: KeywordCompletionFilters,
    pub literals: Vec<LiteralValue>,
    // PORT: key is the index in `symbols` (Go `int`).
    pub symbol_to_origin_info_map: FxHashMap<i32, SymbolOriginInfo>,
    // PORT: key is Go `ast.SymbolId` (`get_symbol_id`).
    pub symbol_to_sort_text_map: FxHashMap<u64, SortText>,
    pub recommended_completion: SymbolId,
    pub previous_token: Node,
    pub context_token: Node,
    pub jsx_initializer: JsxInitializer,
    pub inside_js_doc_tag_type_expression: bool,
    pub is_type_only_location: bool,
    // In JSX tag name and attribute names, identifiers like "my-tag" or "aria-name" is valid identifier.
    pub is_jsx_identifier_expected: bool,
    pub is_right_of_open_tag: bool,
    pub is_right_of_dot_or_question_dot: bool,
    pub import_statement_completion: Option<ImportStatementCompletionInfo>, // !!!
    pub has_unresolved_auto_imports: bool,                                  // !!!
    // flags CompletionInfoFlags // !!!
    // PORT: Go keeps nil and empty slices apart here; `None` is nil.
    pub default_commit_characters: Option<Vec<String>>,
}

// Go: ls/completions.go:195 completionDataKeyword

// Go: ls/completions.go:200 completionDataJSDocTagName
#[derive(Clone, Copy, Debug, Default)]
pub struct CompletionDataJSDocTagName;

// Go: ls/completions.go:202 completionDataJSDocTag
#[derive(Clone, Copy, Debug, Default)]
pub struct CompletionDataJSDocTag;

// Go: ls/completions.go:204 completionDataJSDocParameterName
#[derive(Clone, Copy, Debug, Default)]
pub struct CompletionDataJSDocParameterName {
    pub tag: Node,
}

// Go: ls/completions.go:208 importStatementCompletionInfo
#[derive(Clone, Debug)]
pub struct ImportStatementCompletionInfo {
    pub is_keyword_only_completion: bool,
    pub keyword_completion: SyntaxKind, // TokenKind
    pub is_new_identifier_location: bool,
    pub is_top_level_type_only: bool,
    pub could_be_type_only_import_specifier: bool,
    pub replacement_span: Option<lsproto::Range>,
}

// PORT: the Go zero value has `keywordCompletion == ast.KindUnknown`.
impl Default for ImportStatementCompletionInfo {
    fn default() -> Self {
        ImportStatementCompletionInfo {
            is_keyword_only_completion: false,
            keyword_completion: SyntaxKind::Unknown,
            is_new_identifier_location: false,
            is_top_level_type_only: false,
            could_be_type_only_import_specifier: false,
            replacement_span: None,
        }
    }
}

// Go: ls/completions.go:219 jsxInitializer
// If we're after the `=` sign but no identifier has been typed yet,
// value will be `true` but initializer will be `nil`.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsxInitializer {
    pub is_initializer: bool,
    pub initializer: Node,
}

// Go: ls/completions.go:224 KeywordCompletionFilters
go_enum!(KeywordCompletionFilters, i32 {
    NONE = 0; // No keywords
    ALL = 1; // Every possible kewyord
    CLASS_ELEMENT_KEYWORDS = 2; // Keywords inside class body
    INTERFACE_ELEMENT_KEYWORDS = 3; // Keywords inside interface body
    CONSTRUCTOR_PARAMETER_KEYWORDS = 4; // Keywords at constructor parameter
    FUNCTION_LIKE_BODY_KEYWORDS = 5; // Keywords at function like body
    TYPE_ASSERTION_KEYWORDS = 6;
    TYPE_KEYWORDS = 7;
    TYPE_KEYWORD = 8; // Literally just `type`
    LAST = 8;
});

// Go: ls/completions.go:239 keywordFiltersFromSyntaxKind
// PORT: Go `ast.Kind.String()` is "Kind" + the kind name; the Rust `Debug`
// name spells JSDoc kinds `JsDoc`.
pub fn keyword_filters_from_syntax_kind(
    keyword_completion: SyntaxKind,
) -> KeywordCompletionFilters {
    match keyword_completion {
        SyntaxKind::TypeKeyword => KeywordCompletionFilters::TYPE_KEYWORD,
        _ => crate::core::go_panic(format!(
            "Unknown mapping from ast.Kind `{}` to KeywordCompletionFilters",
            crate::gostd::debug::kind_string(keyword_completion)
        )),
    }
}

// Go: ls/completions.go:248 CompletionKind
go_enum!(CompletionKind, i32 {
    NONE = 0;
    OBJECT_PROPERTY_DECLARATION = 1;
    GLOBAL = 2;
    PROPERTY_ACCESS = 3;
    MEMBER_LIKE = 4;
    STRING = 5;
});

// Go: ls/completions.go:259 CompletionTriggerCharacters
pub static COMPLETION_TRIGGER_CHARACTERS: [&str; 10] =
    [".", "\"", "'", "`", "/", "@", "<", "#", " ", "*"];

// Go: ls/completions.go:262 allCommitCharacters
// All commit characters, valid when `isNewIdentifierLocation` is false.
pub static ALL_COMMIT_CHARACTERS: &[&str] = &[".", ",", ";"];

// Go: ls/completions.go:265 noCommaCommitCharacters
// Commit characters valid at expression positions where we could be inside a parameter list.
pub static NO_COMMA_COMMIT_CHARACTERS: &[&str] = &[".", ";"];

// Go: ls/completions.go:267 emptyCommitCharacters
pub static EMPTY_COMMIT_CHARACTERS: &[&str] = &[];

// Go: ls/completions.go:269 SortText
pub type SortText = String;

// Go: ls/completions.go:269 SortText consts
pub const SORT_TEXT_LOCAL_DECLARATION_PRIORITY: &str = "10";
pub const SORT_TEXT_LOCATION_PRIORITY: &str = "11";
pub const SORT_TEXT_OPTIONAL_MEMBER: &str = "12";
pub const SORT_TEXT_MEMBER_DECLARED_BY_SPREAD_ASSIGNMENT: &str = "13";
pub const SORT_TEXT_SUGGESTED_CLASS_MEMBERS: &str = "14";
pub const SORT_TEXT_GLOBALS_OR_KEYWORDS: &str = "15";
pub const SORT_TEXT_AUTO_IMPORT_SUGGESTIONS: &str = "16";
pub const SORT_TEXT_CLASS_MEMBER_SNIPPETS: &str = "17";
pub const SORT_TEXT_JAVASCRIPT_IDENTIFIERS: &str = "18";

// Go: ls/completions.go:283 DeprecateSortText
pub fn deprecate_sort_text(original: &str) -> SortText {
    format!("z{original}")
}

// Go: ls/completions.go:287 ObjectLiteralPropertySortText
pub fn object_literal_property_sort_text(
    preset_sort_text: &str,
    symbol_display_name: &str,
) -> SortText {
    format!("{preset_sort_text}\x00{symbol_display_name}\x00")
}

// Go: ls/completions.go:291 SortBelow
pub fn sort_below(original: &str) -> SortText {
    format!("{original}1")
}

// Go: ls/completions.go:295 symbolOriginInfoKind
go_flags!(SymbolOriginInfoKind, i32 {
    THIS_TYPE = 1; // 1 << iota
    SYMBOL_MEMBER = 2;
    PROMISE = 4;
    NULLABLE = 8;
    TYPE_ONLY_ALIAS = 16;
    OBJECT_LITERAL_METHOD = 32;
    IGNORE = 64;
    COMPUTED_PROPERTY_NAME = 128;
});

// Go: ls/completions.go:308 symbolOriginInfo
#[derive(Clone, Debug, Default)]
pub struct SymbolOriginInfo {
    pub kind: SymbolOriginInfoKind,
    pub is_default_export: bool,
    pub is_from_package_json: bool,
    pub file_name: String,
    pub data: SymbolOriginInfoData,
}

// PORT: Go `data any` holds nil or one of three pointer types.
#[derive(Clone, Debug, Default)]
pub enum SymbolOriginInfoData {
    #[default]
    None,
    ObjectLiteralMethod(SymbolOriginInfoObjectLiteralMethod),
    TypeOnlyAlias(SymbolOriginInfoTypeOnlyAlias),
    ComputedPropertyName(SymbolOriginInfoComputedPropertyName),
}

impl SymbolOriginInfoData {
    /// Go `%T` of `symbolOriginInfo.data`, for the Go panic texts.
    fn go_type_name(&self) -> &'static str {
        match self {
            SymbolOriginInfoData::None => "<nil>",
            SymbolOriginInfoData::ObjectLiteralMethod(_) => {
                "*ls.symbolOriginInfoObjectLiteralMethod"
            }
            SymbolOriginInfoData::TypeOnlyAlias(_) => "*ls.symbolOriginInfoTypeOnlyAlias",
            SymbolOriginInfoData::ComputedPropertyName(_) => {
                "*ls.symbolOriginInfoComputedPropertyName"
            }
        }
    }
}

impl SymbolOriginInfo {
    // Go: ls/completions.go:242 (*symbolOriginInfo).symbolName
    pub fn symbol_name(&self) -> String {
        match &self.data {
            SymbolOriginInfoData::ComputedPropertyName(data) => data.symbol_name.clone(),
            _ => crate::core::go_panic(format!(
                "symbolOriginInfo: unknown data type for symbolName(): {}",
                self.data.go_type_name()
            )),
        }
    }

    // Go: ls/completions.go:257 (*symbolOriginInfo).asObjectLiteralMethod
    // PORT: a failed Go type assertion panics; the text follows the Go runtime.
    pub fn as_object_literal_method(&self) -> &SymbolOriginInfoObjectLiteralMethod {
        match &self.data {
            SymbolOriginInfoData::ObjectLiteralMethod(data) => data,
            _ => crate::core::go_panic(format!(
                "interface conversion: interface {{}} is {}, not *ls.symbolOriginInfoObjectLiteralMethod",
                self.data.go_type_name()
            )),
        }
    }
}

// Go: ls/completions.go:325 symbolOriginInfoObjectLiteralMethod
#[derive(Clone, Debug, Default)]
pub struct SymbolOriginInfoObjectLiteralMethod {
    pub insert_text: String,
    pub label_details: Option<lsproto::CompletionItemLabelDetails>,
    pub is_snippet: bool,
}

// Go: ls/completions.go:335 symbolOriginInfoTypeOnlyAlias
#[derive(Clone, Copy, Debug, Default)]
pub struct SymbolOriginInfoTypeOnlyAlias {
    pub declaration: Node,
}

// Go: ls/completions.go:339 symbolOriginInfoComputedPropertyName
#[derive(Clone, Debug, Default)]
pub struct SymbolOriginInfoComputedPropertyName {
    pub symbol_name: String,
}

// Go: ls/completions.go:352 completionSource
// Special values for `CompletionInfo['source']` used to disambiguate
// completion items with the same `name`. (Each completion item must
// have a unique name/source combination, because those two fields
// comprise `CompletionEntryIdentifier` in `getCompletionEntryDetails`.
//
// When the completion item is an auto-import suggestion, the source
// is the module specifier of the suggestion. To avoid collisions,
// the values here should not be a module specifier we would ever
// generate for an auto-import.
pub type CompletionSource = &'static str;

// Completions that require `this.` insertion text.
pub const COMPLETION_SOURCE_THIS_PROPERTY: CompletionSource = "ThisProperty/";
// Auto-import that comes attached to a class member snippet.
pub const COMPLETION_SOURCE_CLASS_MEMBER_SNIPPET: CompletionSource = "ClassMemberSnippet/";
// A type-only import that needs to be promoted in order to be used at the completion location.
pub const COMPLETION_SOURCE_TYPE_ONLY_ALIAS: CompletionSource = "TypeOnlyAlias/";
// Auto-import that comes attached to an object literal method snippet.
pub const COMPLETION_SOURCE_OBJECT_LITERAL_METHOD_SNIPPET: CompletionSource =
    "ObjectLiteralMethodSnippet/";
// Case completions for switch statements.
pub const COMPLETION_SOURCE_SWITCH_CASES: CompletionSource = "SwitchCases/";
// Completions for an object literal expression.
pub const COMPLETION_SOURCE_OBJECT_LITERAL_MEMBER_WITH_COMMA: CompletionSource =
    "ObjectLiteralMemberWithComma/";

// Go: ls/completions.go:371 uniqueNamesMap
// Value is set to false for global variables or completions from external module exports,
// true otherwise.
pub type UniqueNamesMap = FxHashMap<String, bool>;

// Go: ls/completions.go:374 literalValue
// PORT: Go `literalValue any` (string | jsnum.Number | PseudoBigInt) is
// `LiteralValue` from checker/types.rs; Go nil is `None`.

// Go: ls/completions.go:376 globalsSearch
go_enum!(GlobalsSearch, i32 {
    CONTINUE = 0;
    SUCCESS = 1;
    FAIL = 2;
});





