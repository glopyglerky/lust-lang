use crate::ls::prelude::*;

// Port of Go `ls/completions.go` lines 1709-3498: completion entry building,
// keyword lists and the context helpers.
//
// PORT (whole file):
// - Go text scanning (`utf8.DecodeRuneInString`, `utf8.DecodeLastRuneInString`,
//   `unicode.IsSpace`, `unicode.IsDigit`) runs on byte offsets with the
//   scanner's rune decoders. A Go `rune` is an `i32`.
// - Go `*ast.Symbol` reads without a checker take the symbol arena as the
//   first parameter (`symbols: &SymbolArena`), as the ast helpers do.
// - Go `*checker.Type` is `TypeId`; its methods read the checker arena
//   (`type_checker.ty(t)`).
// - Go `*symbolOriginInfo` parameters are `Option<&SymbolOriginInfo>`.
// - Go `collections.Set[string]` results are `FxHashSet<String>`.

use crate::astnav;
use crate::frontend::json_ext;
use crate::frontend::scanner::scanner_p1::{
    RUNE_ERROR, 
};
use crate::frontend::stringutil_ls;
use crate::gostd::{GoError, unicode};
use crate::ls::lsutil;
use crate::lsp::lsproto;

// Go: ls/completions.go:1802 keywordCompletionData

// Go: ls/completions.go:1813 getDefaultCommitCharacters
pub fn get_default_commit_characters(is_new_identifier_location: bool) -> Vec<String> {
    if is_new_identifier_location {
        return Vec::new();
    }
    ALL_COMMIT_CHARACTERS
        .iter()
        .map(|s| (*s).to_string())
        .collect()
}


// Go: ls/completions.go:2126 completionNameForLiteral

// Go: ls/completions.go:2143 getInsertTextAndReplacementSpanForImportCompletion

// Go: ls/completions.go:2165 createCompletionItemForLiteral


// Go: ls/completions.go:2467 memberCompletionEntry
#[derive(Clone, Debug, Default)]
pub struct MemberCompletionEntry {
    pub insert_text: String,
    pub filter_text: String,
    pub is_snippet: bool,
    pub additional_text_edits: Vec<lsproto::TextEdit>,
}


// Go: ls/completions.go:2573 isObjectLiteralMethodCompletionCandidateDeclaration

// Go: ls/completions.go:2585 objectLiteralMethodSymbol
// PORT: Go `origin *symbolOriginInfo` is never nil here; it is held by value.
#[derive(Clone, Debug, Default)]
pub struct ObjectLiteralMethodSymbol {
    pub symbol: SymbolId,
    pub origin: SymbolOriginInfo,
}


// Go: ls/completions.go:2620 isObjectLiteralMethodSymbol
// PORT: Go reads `symbol.Flags` through the pointer; `symbols` is the arena
// that holds `symbol`.


// Go: ls/completions.go:2760 presentMemberModifiers
#[derive(Clone, Debug, Default)]
pub struct PresentMemberModifiers {
    pub modifiers: ModifierFlags,
    pub decorators: Vec<Node>,
    pub erase_range: Option<lsproto::Range>,
}


// Go: ls/completions.go:2819 modifierLikeKind

// Go: ls/completions.go:2835 createModifierList
// PORT: Go returns a nil `*ast.ModifierList` when there are no nodes; that is
// `ModifierList::NIL`.

// Go: ls/completions.go:2847 createSnippetTabStopBody


// Go: ls/completions.go:2870 isRecommendedCompletionMatch

// Go: ls/completions.go:2876 wordSeparators
// Ported from vscode.
// PORT: Go `collections.Set[rune]`; a rune is an `i32`.
pub static WORD_SEPARATORS: [i32; 29] = [
    '`' as i32,
    '~' as i32,
    '!' as i32,
    '@' as i32,
    '%' as i32,
    '^' as i32,
    '&' as i32,
    '*' as i32,
    '(' as i32,
    ')' as i32,
    '-' as i32,
    '=' as i32,
    '+' as i32,
    '[' as i32,
    '{' as i32,
    ']' as i32,
    '}' as i32,
    '\\' as i32,
    '|' as i32,
    ';' as i32,
    ':' as i32,
    '\'' as i32,
    '"' as i32,
    ',' as i32,
    '.' as i32,
    '<' as i32,
    '>' as i32,
    '/' as i32,
    '?' as i32,
];

// Go: ls/completions.go:2883 getWordLengthAndStart
// Finds the length and first rune of the word that ends at the given position.
// e.g. for "abc def.ghi|jkl", the word length is 3 and the word start is 'g'.
// PORT: Go cuts `text := sourceFile.Text()[:position]`. The decoders read the
// whole text and stop at `position`, so no `&str` is cut inside a character.
// A position past the text panics there with Go's text
// (`go_check_slice_bounds`): the API in an LSP server with JSDoc completions
// off reaches it (with them on, the JSDoc snippet check panics first).

// Go: ls/completions.go:2906 trimElementAccess
// `["ab c"]` -> `ab c`
// `['ab c']` -> `ab c`
// `[123]` -> `123`
pub fn trim_element_access(text: &str) -> String {
    let mut text = text.strip_prefix('[').unwrap_or(text);
    text = text.strip_suffix(']').unwrap_or(text);
    if text.starts_with('\'') && text.ends_with('\'') {
        let trimmed = text.strip_suffix('\'').unwrap_or(text);
        text = trimmed.strip_prefix('\'').unwrap_or(trimmed);
    }
    if text.starts_with('"') && text.ends_with('"') {
        let trimmed = text.strip_suffix('"').unwrap_or(text);
        text = trimmed.strip_prefix('"').unwrap_or(trimmed);
    }
    text.to_string()
}

// Go: ls/completions.go:2919 getFilterText
// Ported from vscode ts extension: `getFilterText`.
pub fn get_filter_text(
    file: Node,
    position: i32,
    insert_text: &str,
    label: &str,
    word_start: i32,
    dot_accessor: &str,
) -> String {
    // Private field completion, e.g. label `#bar`.
    if let Some(after) = label.strip_prefix('#') {
        if !insert_text.is_empty() {
            if let Some(after) = insert_text.strip_prefix("this.#") {
                if word_start == '#' as i32 {
                    // `method() { this.#| }`
                    // `method() { #| }`
                    return String::new();
                } else {
                    // `method() { this.| }`
                    // `method() { | }`
                    return after.to_string();
                }
            }
        } else if word_start == '#' as i32 {
            // `method() { this.#| }`
            return String::new();
        } else {
            // `method() { this.| }`
            // `method() { | }`
            return after.to_string();
        }
    }

    // For `this.` completions, generally don't set the filter text since we don't want them to be overly deprioritized. microsoft/vscode#74164
    if insert_text.starts_with("this.") {
        return String::new();
    }

    // Handle the case:
    // ```
    // const xyz = { 'ab c': 1 };
    // xyz.ab|
    // ```
    // In which case we want to insert a bracket accessor but should use `.abc` as the filter text instead of
    // the bracketed insert text.
    if insert_text.starts_with('[') {
        return dot_accessor.to_string() + &trim_element_access(insert_text);
    }

    if insert_text.starts_with("?.") {
        // Handle this case like the case above:
        // ```
        // const xyz = { 'ab c': 1 } | undefined;
        // xyz.ab|
        // ```
        // filterText should be `.ab c` instead of `?.['ab c']`.
        if insert_text.starts_with("?.[") {
            return dot_accessor.to_string() + &trim_element_access(&insert_text[2..]);
        } else {
            // ```
            // const xyz = { abc: 1 } | undefined;
            // xyz.ab|
            // ```
            // filterText should be `.abc` instead of `?.abc.
            return dot_accessor.to_string() + &insert_text[2..];
        }
    }

    // In all other cases, fall back to using the insertText.
    insert_text.to_string()
}

// Go: ls/completions.go:2993 getDotAccessor
// Ported from vscode's `provideCompletionItems`.

// Go: ls/completions.go:3007 strPtrIsEmpty
pub fn str_ptr_is_empty(ptr: Option<String>) -> bool {
    match ptr {
        None => true,
        Some(ptr) => ptr.is_empty(),
    }
}

// Go: ls/completions.go:3014 strPtrTo
pub fn str_ptr_to(v: &str) -> Option<String> {
    if v.is_empty() {
        return None;
    }
    Some(v.to_string())
}

// Go: ls/completions.go:3021 boolToPtr
pub fn bool_to_ptr(v: bool) -> Option<bool> {
    if v {
        return Some(true);
    }
    None
}

// Go: ls/completions.go:3028 getLineOfPosition

// Go: ls/completions.go:3033 getLineEndOfPosition

// Go: ls/completions.go:3049 isClassLikeMemberCompletion
// PORT: Go reads `symbol.Flags` through the pointer; `symbols` is the arena
// that holds `symbol`.

// Go: ls/completions.go:3060 symbolAppearsToBeTypeOnly

// Go: ls/completions.go:3066 shouldIncludeSymbol

// Go: ls/completions.go:3158 getCompletionEntryDisplayNameForSymbol
// PORT: `checker.IsKnownSymbol(symbol)` is `isLateBoundName(symbol.Name)`;
// it reads the arena directly (the port has it as a `Checker` method).

// !!! refactor symbolOriginInfo so that we can tell the difference between flags and the kind of data it has
// Go: ls/completions.go:3220 originIsIgnore
pub fn origin_is_ignore(origin: Option<&SymbolOriginInfo>) -> bool {
    origin.is_some_and(|origin| origin.kind.intersects(SymbolOriginInfoKind::IGNORE))
}

// Go: ls/completions.go:3224 originIncludesSymbolName
pub fn origin_includes_symbol_name(origin: Option<&SymbolOriginInfo>) -> bool {
    origin_is_computed_property_name(origin)
}

// Go: ls/completions.go:3228 originIsComputedPropertyName
pub fn origin_is_computed_property_name(origin: Option<&SymbolOriginInfo>) -> bool {
    origin.is_some_and(|origin| {
        origin
            .kind
            .intersects(SymbolOriginInfoKind::COMPUTED_PROPERTY_NAME)
    })
}

// Go: ls/completions.go:3232 originIsObjectLiteralMethod
pub fn origin_is_object_literal_method(origin: Option<&SymbolOriginInfo>) -> bool {
    origin.is_some_and(|origin| {
        origin
            .kind
            .intersects(SymbolOriginInfoKind::OBJECT_LITERAL_METHOD)
    })
}

// Go: ls/completions.go:3236 originIsThisTypeNode
pub fn origin_is_this_type_node(origin: Option<&SymbolOriginInfo>) -> bool {
    origin.is_some_and(|origin| origin.kind.intersects(SymbolOriginInfoKind::THIS_TYPE))
}

// Go: ls/completions.go:3240 originIsTypeOnlyAlias
pub fn origin_is_type_only_alias(origin: Option<&SymbolOriginInfo>) -> bool {
    origin.is_some_and(|origin| {
        origin
            .kind
            .intersects(SymbolOriginInfoKind::TYPE_ONLY_ALIAS)
    })
}

// Go: ls/completions.go:3244 originIsSymbolMember
pub fn origin_is_symbol_member(origin: Option<&SymbolOriginInfo>) -> bool {
    origin.is_some_and(|origin| origin.kind.intersects(SymbolOriginInfoKind::SYMBOL_MEMBER))
}

// Go: ls/completions.go:3248 originIsNullableMember
pub fn origin_is_nullable_member(origin: Option<&SymbolOriginInfo>) -> bool {
    origin.is_some_and(|origin| origin.kind.intersects(SymbolOriginInfoKind::NULLABLE))
}

// Go: ls/completions.go:3252 originIsPromise
pub fn origin_is_promise(origin: Option<&SymbolOriginInfo>) -> bool {
    origin.is_some_and(|origin| origin.kind.intersects(SymbolOriginInfoKind::PROMISE))
}

// Go: ls/completions.go:3256 getSourceFromOrigin
pub fn get_source_from_origin(origin: Option<&SymbolOriginInfo>) -> String {
    if origin_is_this_type_node(origin) {
        return COMPLETION_SOURCE_THIS_PROPERTY.to_string();
    }

    if origin_is_type_only_alias(origin) {
        return COMPLETION_SOURCE_TYPE_ONLY_ALIAS.to_string();
    }

    String::new()
}

// Go: ls/completions.go:3271 getRelevantTokens
// In a scenarion such as `const x = 1 * |`, the context and previous tokens are both `*`.
// In `const x = 1 * o|`, the context token is *, and the previous token is `o`.
// `contextToken` and `previousToken` can both be nil if we are at the beginning of the file.
// PORT: returns `(contextToken, previousToken)`.

// Go: ls/completions.go:3281 CompletionsTriggerCharacter
// "." | '"' | "'" | "`" | "/" | "@" | "<" | "#" | " " | "*"
pub type CompletionsTriggerCharacter = String;

// Go: ls/completions.go:3283 isValidTrigger
// PORT: the `CompletionsTriggerCharacter` (Go string) parameter is `&str`.

// Go: ls/completions.go:3318 isStringLiteralOrTemplate

// Go: ls/completions.go:3327 binaryExpressionMayBeOpenTag

// Go: ls/completions.go:3331 isCheckedFile

// Go: ls/completions.go:3335 isContextTokenValueLocation

// Go: ls/completions.go:3341 isPossiblyTypeArgumentPosition

// Go: ls/completions.go:3348 isContextTokenTypeLocation

/// Go `collections.Set[ast.SymbolId]` passed by value
/// (`symbolCanBeReferencedAtTypeLocation`, completions.go:2792).
// PORT: Go copies the `Set` struct on each call. A copy shares the map once
// the map exists; `Add` on a nil map makes a new map in that copy only.
// `None` is the nil map; a clone of `Some` shares the one map. Callers that
// pass Go's `collections.Set[ast.SymbolId]{}` pass `None`.
pub type SymbolIdSetValue = Option<Rc<RefCell<FxHashSet<u64>>>>;

// Go: collections/set.go:57 (*Set).AddIfAbsent, on a by-value `Set` copy.

// Go: ls/completions.go:3376 symbolCanBeReferencedAtTypeLocation
// True if symbol is a type or a module containing at least one type.

// Go: ls/completions.go:3387 nonAliasCanBeReferencedAtTypeLocation

// Go: core/core.go:706 CheckEachDefined
fn check_each_defined(s: Vec<SymbolId>, msg: &str) -> Vec<SymbolId> {
    for value in &s {
        if value.is_nil() {
            crate::core::go_panic(msg.to_string());
        }
    }
    s
}

// Go: ls/completions.go:3398 getPropertiesForCompletion
// Gets all properties on a type, but if that type is a union of several types,
// excludes array-like types or callable/constructable types.

// Go: ls/completions.go:3407 getLeftMostName
// Given 'a.b.c', returns 'a'.

// Go: ls/completions.go:3417 getFirstSymbolInChain

// Go: ls/completions.go:3436 isModuleSymbol

// Go: ls/completions.go:3440 getNullableSymbolOriginInfoKind
pub fn get_nullable_symbol_origin_info_kind(
    kind: SymbolOriginInfoKind,
    insert_question_dot: bool,
) -> SymbolOriginInfoKind {
    let mut kind = kind;
    if insert_question_dot {
        kind |= SymbolOriginInfoKind::NULLABLE;
    }
    kind
}

// Go: ls/completions.go:3447 isStaticProperty

// Go: ls/completions.go:3455 getContextualTypeForConditionalExpression
// getContextualTypeForConditionalExpression handles completion within a conditional expression
// (ternary operator) by using the parent expression to find the contextual type.

// Go: ls/completions.go:3468 getContextualType

// Go: ls/completions.go:3557 getSwitchedType

// Go: ls/completions.go:3561 isEqualityOperatorKind
pub fn is_equality_operator_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::EqualsEqualsEqualsToken
            | SyntaxKind::EqualsEqualsToken
            | SyntaxKind::ExclamationEqualsEqualsToken
            | SyntaxKind::ExclamationEqualsToken
    )
}

// Go: ls/completions.go:3572 isLiteral
// We disregard boolean literals for completion purposes.
// PORT: Go reads the `*checker.Type`; the port reads the checker arena.

// Go: ls/completions.go:3576 getRecommendedCompletion

// Go: ls/completions.go:3599 isAbstractConstructorSymbol

// Go: ls/completions.go:3607 startsWithQuote

// Go: ls/completions.go:3612 getClosestSymbolDeclaration

// Go: ls/completions.go:3644 isArrowFunctionBody

// Go: ls/completions.go:3651 isInTypeParameterDefault

// Go: ls/completions.go:3669 isDeprecated


// Go: ls/completions.go:3709 quotePropertyName

// Go: ls/completions.go:3720 isStringAndEmptyAnonymousObjectIntersection
// Checks whether type is `string & {}`, which is semantically equivalent to string but
// is not reduced by the checker as a special case used for supporting string literal completions
// for string type.

// Go: ls/completions.go:3730 areIntersectedTypesAvoidingStringReduction

// Go: ls/completions.go:3734 escapeSnippetText
pub fn escape_snippet_text(text: &str) -> String {
    text.replace('$', "\\$")
}

// Go: ls/completions.go:3738 isNamedImportsOrExports

// Go: ls/completions.go:3742 generateIdentifierForArbitraryString

// Go: ls/completions.go:3782 getCompletionsSymbolKind
// Copied from vscode TS extension.

// Go: ls/completions.go:3837 CompareCompletionEntries
// Editors will use the `sortText` and then fall back to `name` for sorting, but leave ties in response order.
// So, it's important that we sort those ties in the order we want them displayed if it matters. We don't
// strictly need to sort by name or SortText here since clients are going to do it anyway, but we have to
// do the work of comparing them so we can sort those ties appropriately.
// PORT: Go dereferences `SortText`; a nil pointer panics there.


// Go: ls/completions.go:3848 allKeywordCompletions
// PORT: Go returns the shared slice; the port returns a copy. Callers copy
// every item before they change it (`cloneItems`).

// Go: ls/completions.go:3861 cloneItems
// PORT: Go returns nil for a nil input; a `Vec` has no nil, so both are empty.

// Go: ls/completions.go:3873 getKeywordCompletions

// Go: ls/completions.go:3892 getTypescriptKeywordCompletions

// Go: ls/completions.go:3932 isTypeScriptOnlyKeyword
pub fn is_type_script_only_keyword(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::AbstractKeyword
            | SyntaxKind::AnyKeyword
            | SyntaxKind::BigIntKeyword
            | SyntaxKind::BooleanKeyword
            | SyntaxKind::DeclareKeyword
            | SyntaxKind::EnumKeyword
            | SyntaxKind::GlobalKeyword
            | SyntaxKind::ImplementsKeyword
            | SyntaxKind::InferKeyword
            | SyntaxKind::InterfaceKeyword
            | SyntaxKind::IsKeyword
            | SyntaxKind::KeyOfKeyword
            | SyntaxKind::ModuleKeyword
            | SyntaxKind::NamespaceKeyword
            | SyntaxKind::NeverKeyword
            | SyntaxKind::NumberKeyword
            | SyntaxKind::ObjectKeyword
            | SyntaxKind::OverrideKeyword
            | SyntaxKind::PrivateKeyword
            | SyntaxKind::ProtectedKeyword
            | SyntaxKind::PublicKeyword
            | SyntaxKind::ReadonlyKeyword
            | SyntaxKind::StringKeyword
            | SyntaxKind::SymbolKeyword
            | SyntaxKind::TypeKeyword
            | SyntaxKind::UniqueKeyword
            | SyntaxKind::UnknownKeyword
    )
}

// Go: ls/completions.go:3967 isFunctionLikeBodyKeyword
pub fn is_function_like_body_keyword(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::AsyncKeyword
        || kind == SyntaxKind::AwaitKeyword
        || kind == SyntaxKind::UsingKeyword
        || kind == SyntaxKind::AsKeyword
        || kind == SyntaxKind::SatisfiesKeyword
        || kind == SyntaxKind::TypeKeyword
        || !is_contextual_keyword(kind) && !is_class_member_completion_keyword(kind)
}

// Go: ls/completions.go:3977 isClassMemberCompletionKeyword
pub fn is_class_member_completion_keyword(kind: SyntaxKind) -> bool {
    match kind {
        SyntaxKind::AbstractKeyword
        | SyntaxKind::AccessorKeyword
        | SyntaxKind::ConstructorKeyword
        | SyntaxKind::GetKeyword
        | SyntaxKind::SetKeyword
        | SyntaxKind::AsyncKeyword
        | SyntaxKind::DeclareKeyword
        | SyntaxKind::OverrideKeyword => true,
        _ => is_class_member_modifier(kind),
    }
}

// Go: ls/completions.go:3987 isInterfaceOrTypeLiteralCompletionKeyword
pub fn is_interface_or_type_literal_completion_keyword(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::ReadonlyKeyword
}

// Go: ls/completions.go:3991 isContextualKeywordInAutoImportableExpressionSpace
pub fn is_contextual_keyword_in_auto_importable_expression_space(keyword: &str) -> bool {
    keyword == "abstract"
        || keyword == "async"
        || keyword == "await"
        || keyword == "declare"
        || keyword == "module"
        || keyword == "namespace"
        || keyword == "type"
        || keyword == "satisfies"
        || keyword == "as"
}

// Go: ls/completions.go:4003 getContextualKeywords


// Go: ls/completions.go:4069 isMemberCompletionKind
pub fn is_member_completion_kind(kind: CompletionKind) -> bool {
    kind == CompletionKind::OBJECT_PROPERTY_DECLARATION
        || kind == CompletionKind::MEMBER_LIKE
        || kind == CompletionKind::PROPERTY_ACCESS
}

// Go: ls/completions.go:4075 tryGetFunctionLikeBodyCompletionContainer

// ---------------------------------------------------------------------------
// Go `unicode` and `unicode/utf8` helpers used above (file-local).
// ---------------------------------------------------------------------------


/// A decoded Go rune as a `char` (`utf8.RuneError` is U+FFFD).
fn rune_to_char(r: i32) -> char {
    char::from_u32(r as u32).unwrap_or(char::REPLACEMENT_CHARACTER)
}

