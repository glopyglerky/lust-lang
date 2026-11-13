use crate::ls::prelude::*;

// Go `internal/ls/semantictokens.go`: textDocument/semanticTokens.

use crate::flags_macros::{go_enum, go_flags};
use crate::spanmap::{Feature, Fidelity};

// Go: ls/semantictokens.go:21 tokenTypes
// tokenTypes defines the order of token types for encoding
static TOKEN_TYPES: [lsproto::SemanticTokenType; 23] = [
    lsproto::SemanticTokenType::NAMESPACE,
    lsproto::SemanticTokenType::CLASS,
    lsproto::SemanticTokenType::ENUM,
    lsproto::SemanticTokenType::INTERFACE,
    lsproto::SemanticTokenType::STRUCT,
    lsproto::SemanticTokenType::TYPE_PARAMETER,
    lsproto::SemanticTokenType::TYPE,
    lsproto::SemanticTokenType::PARAMETER,
    lsproto::SemanticTokenType::VARIABLE,
    lsproto::SemanticTokenType::PROPERTY,
    lsproto::SemanticTokenType::ENUM_MEMBER,
    lsproto::SemanticTokenType::DECORATOR,
    lsproto::SemanticTokenType::EVENT,
    lsproto::SemanticTokenType::FUNCTION,
    lsproto::SemanticTokenType::METHOD,
    lsproto::SemanticTokenType::MACRO,
    lsproto::SemanticTokenType::LABEL,
    lsproto::SemanticTokenType::COMMENT,
    lsproto::SemanticTokenType::STRING,
    lsproto::SemanticTokenType::KEYWORD,
    lsproto::SemanticTokenType::NUMBER,
    lsproto::SemanticTokenType::REGEXP,
    lsproto::SemanticTokenType::OPERATOR,
];

// Go: ls/semantictokens.go:48 tokenModifiers
// tokenModifiers defines the order of token modifiers for encoding
static TOKEN_MODIFIERS: [lsproto::SemanticTokenModifier; 11] = [
    lsproto::SemanticTokenModifier::DECLARATION,
    lsproto::SemanticTokenModifier::DEFINITION,
    lsproto::SemanticTokenModifier::READONLY,
    lsproto::SemanticTokenModifier::STATIC,
    lsproto::SemanticTokenModifier::DEPRECATED,
    lsproto::SemanticTokenModifier::ABSTRACT,
    lsproto::SemanticTokenModifier::ASYNC,
    lsproto::SemanticTokenModifier::MODIFICATION,
    lsproto::SemanticTokenModifier::DOCUMENTATION,
    lsproto::SemanticTokenModifier::DEFAULT_LIBRARY,
    lsproto::SemanticTokenModifier(std::borrow::Cow::Borrowed("local")),
];

// Go: ls/semantictokens.go:62 tokenType
go_enum!(TokenType, i32 {
    NAMESPACE = 0;
    CLASS = 1;
    ENUM = 2;
    INTERFACE = 3;
    STRUCT = 4;
    TYPE_PARAMETER = 5;
    TYPE = 6;
    PARAMETER = 7;
    VARIABLE = 8;
    PROPERTY = 9;
    ENUM_MEMBER = 10;
    DECORATOR = 11;
    EVENT = 12;
    FUNCTION = 13;
    METHOD = 14; // Previously called "member" in TypeScript
    MACRO = 15;
    LABEL = 16;
    COMMENT = 17;
    STRING = 18;
    KEYWORD = 19;
    NUMBER = 20;
    REGEXP = 21;
    OPERATOR = 22;
});

// Go: ls/semantictokens.go:90 tokenModifier
go_flags!(TokenModifier, i32 {
    DECLARATION = 1 << 0;
    DEFINITION = 1 << 1;
    READONLY = 1 << 2;
    STATIC = 1 << 3;
    DEPRECATED = 1 << 4;
    ABSTRACT = 1 << 5;
    ASYNC = 1 << 6;
    MODIFICATION = 1 << 7;
    DOCUMENTATION = 1 << 8;
    DEFAULT_LIBRARY = 1 << 9;
    LOCAL = 1 << 10;
});

// Go: ls/semantictokens.go:109 SemanticTokensLegend
// SemanticTokensLegend returns the legend describing the token types and modifiers.
// It filters the legend to only include types and modifiers that the client supports,
// as indicated by clientCapabilities.
// PORT: Go returns a `*lsproto.SemanticTokensLegend` that is never nil; the
// Rust result is `Some` always, the shape of the `legend` field it fills.


// Go: ls/semantictokens.go:193 sortSemanticTokens
// PERF: Go computes both LSP ranges in each comparison. A token's range is
// the same on each call, so its start is computed once here. The comparisons
// give the same results, so the same pdqsort gives the same order.

// Go: ls/semantictokens.go:210 semanticTokenLSPRange

// Go: ls/semantictokens.go:215 semanticToken
// PORT: Go compares the struct by value in the `seen` set of
// ProvideSemanticTokensRange; `Eq` and `Hash` cover all fields, as Go does.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct SemanticToken {
    node: Node,
    file: Node,
    token_type: TokenType,
    token_modifier: TokenModifier,
}

// PORT: the state that the Go closure `visit` in collectSemanticTokensInRange
// captures.


// Go: ls/semantictokens.go:342 classifySymbol
// PORT: symbols live in the checker arena, so it is a parameter (PORTING
// "Names": functions that take a symbol get `symbols: &SymbolArena`).

// Go: ls/semantictokens.go:379 tokenFromDeclarationMapping
fn token_from_declaration_mapping(kind: SyntaxKind) -> TokenType {
    match kind {
        SyntaxKind::VariableDeclaration => TokenType::VARIABLE,
        SyntaxKind::Parameter => TokenType::PARAMETER,
        SyntaxKind::PropertyDeclaration => TokenType::PROPERTY,
        SyntaxKind::ModuleDeclaration => TokenType::NAMESPACE,
        SyntaxKind::EnumDeclaration => TokenType::ENUM,
        SyntaxKind::EnumMember => TokenType::ENUM_MEMBER,
        SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => TokenType::CLASS,
        SyntaxKind::MethodDeclaration => TokenType::METHOD,
        SyntaxKind::FunctionDeclaration | SyntaxKind::FunctionExpression => TokenType::FUNCTION,
        SyntaxKind::MethodSignature => TokenType::METHOD,
        SyntaxKind::GetAccessor | SyntaxKind::SetAccessor => TokenType::PROPERTY,
        SyntaxKind::PropertySignature => TokenType::PROPERTY,
        SyntaxKind::InterfaceDeclaration => TokenType::INTERFACE,
        SyntaxKind::TypeAliasDeclaration => TokenType::TYPE,
        SyntaxKind::TypeParameter => TokenType::TYPE_PARAMETER,
        SyntaxKind::PropertyAssignment | SyntaxKind::ShorthandPropertyAssignment => {
            TokenType::PROPERTY
        }
        _ => TokenType(-1),
    }
}

// Go: ls/semantictokens.go:418 reclassifyByType

// Go: ls/semantictokens.go:464 isLocalDeclaration

// Go: ls/semantictokens.go:489 getDeclarationForBindingElement

// Go: ls/semantictokens.go:504 isInImportClause

// Go: ls/semantictokens.go:509 isExpressionInCallExpression

// Go: ls/semantictokens.go:517 isInfinityOrNaNString
// PORT: the ls-local helper (no "-Infinity"); it shadows the checker helper
// of the same name, as the Go package-level function does.
fn is_infinity_or_nan_string(text: &str) -> bool {
    text == "Infinity" || text == "NaN"
}

// Go: ls/semantictokens.go:523 encodeSemanticTokens
// encodeSemanticTokens encodes tokens into the LSP format using relative positioning.
// It filters tokens based on client capabilities, only including types and modifiers that the client supports.
