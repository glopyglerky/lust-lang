use crate::ls::prelude::*;

// Port of Go `ls/completions.go` lines 3499-4878: commit characters,
// scopes, object, class and JSX completion containers, member filters,
// completion item defaults, label and JSX closing tag completions,
// `createLSPCompletionItem`, completion list blockers, client capability
// checks, argument info for completions and the `Source*` constants.


// Go: completions.go:4094 computeCommitCharactersAndIsNewIdentifier
// PORT: Go returns one of the shared package slices (`allCommitCharacters`,
// `noCommaCommitCharacters`, `emptyCommitCharacters`); Rust returns an owned
// copy. None of them is nil in Go, so the result is a plain `Vec`.

// Go: completions.go:4222 keywordForNode

// Go: completions.go:4231 getScopeNode
// Finds the first node that "embraces" the position, so that one may
// accurately aggregate locals from the closest containing scope.

// Go: completions.go:4239 isSnippetScope

// Go: completions.go:4252 isProbablyGlobalType
// Determines if a type is exactly the same type resolved by the global 'self', 'global', or 'globalThis'.

// Go: completions.go:4270 tryGetTypeLiteralNode

// Go: completions.go:4290 getConstraintOfTypeArgumentProperty

// Go: completions.go:4338 tryGetObjectLikeCompletionContainer

// Go: completions.go:4396 tryGetObjectLiteralContextualType

// Go: completions.go:4417 getPropertiesForObjectExpression

// Go: completions.go:4463 getApparentProperties
// PORT: the ls package function, not the checker method of the same name.

// Go: completions.go:4476 containsNonPublicProperties
// PORT: Go calls the package function `checker.GetDeclarationModifierFlagsFromSymbol`;
// in Rust it is a `Checker` method (symbols live in the checker arena), so
// the checker is an extra first parameter.

// Go: completions.go:4484 filterObjectMembersList
// Filters out members that are already declared in the object literal or binding pattern.
// Also computes the set of existing members declared by spread assignment.

// Go: completions.go:4545 isCurrentlyEditingNode

// Go: completions.go:4550 setMemberDeclaredBySpreadAssignment

// Go: completions.go:4568 tryGetConstructorLikeCompletionContainer
// Returns the immediate owning class declaration of a context token,
// on the condition that one exists and that the context implies completion should be given.

// Go: completions.go:4588 isConstructorParameterCompletion

// Go: completions.go:4595 tryGetObjectTypeDeclarationCompletionContainer
// Returns the immediate owning class declaration of a context token,
// on the condition that one exists and that the context implies completion should be given.

// Go: completions.go:4692 isFromObjectTypeDeclaration

// Go: completions.go:4697 filterClassMembersList
// Filters out completion suggestions for class elements.
// PORT: Go reads the symbols through pointers and calls the package function
// `checker.GetDeclarationModifierFlagsFromSymbol`. In Rust both need the
// checker (symbol arena), so it is an extra first parameter.

// Go: completions.go:4743 tryGetContainingJsxElement

// Go: completions.go:4807 filterJsxAttributes
// Filters out completion suggestions from 'symbols' according to existing JSX attributes.
// @returns Symbols to be suggested in a JSX element, barring those whose attributes
// do not occur at the current position and have not otherwise been typed.
// PORT: Go returns a pointer to the spread member set; it is never nil, so
// Rust returns the set by value.

// Go: completions.go:4833 isTypeKeywordTokenOrIdentifier


// Go: completions.go:5195 isCompletionListBlocker

// Go: completions.go:5210 isInStringOrRegularExpressionOrTemplateLiteral

// Go: completions.go:5222 isSolelyIdentifierDefinitionLocation
// true if we are certain that the currently edited location must define a new location; false otherwise.

// Go: completions.go:5366 isVariableDeclarationListButNotTypeArgument

// Go: completions.go:5371 isFunctionLikeButNotConstructor

// Go: completions.go:5375 isPreviousPropertyDeclarationTerminated

// Go: completions.go:5381 isDotOfNumericLiteral

// Go: completions.go:5391 isInJsxText

// Go: completions.go:5423 clientSupportsItemLabelDetails

// Go: completions.go:5427 clientSupportsItemSnippet

// Go: completions.go:5431 clientSupportsItemCommitCharacters

// Go: completions.go:5435 clientSupportsItemInsertReplace

// Go: completions.go:5439 clientSupportsDefaultCommitCharacters

// Go: completions.go:5443 clientSupportsDefaultEditRange

// Go: completions.go:5447 argumentInfoForCompletions
#[derive(Clone, Copy, Debug, Default)]
pub struct ArgumentInfoForCompletions {
    pub invocation: Node,
    pub argument_index: i32,
    pub argument_count: i32,
}

// Go: completions.go:5453 getArgumentInfoForCompletions

// Go: completions.go:4864 Source* consts
// Special values for `CompletionInfo['source']` used to disambiguate
// completion items with the same `name`. (Each completion item must
// have a unique name/source combination, because those two fields
// comprise `CompletionEntryIdentifier` in `getCompletionEntryDetails`.
//
// When the completion item is an auto-import suggestion, the source
// is the module specifier of the suggestion. To avoid collisions,
// the values here should not be a module specifier we would ever
// generate for an auto-import.

// Completions that require `this.` insertion text
pub const SOURCE_THIS_PROPERTY: &str = "ThisProperty/";
// Auto-import that comes attached to a class member snippet
pub const SOURCE_CLASS_MEMBER_SNIPPET: &str = "ClassMemberSnippet/";
// A type-only import that needs to be promoted in order to be used at the completion location
pub const SOURCE_TYPE_ONLY_ALIAS: &str = "TypeOnlyAlias/";
// Auto-import that comes attached to an object literal method snippet
pub const SOURCE_OBJECT_LITERAL_METHOD_SNIPPET: &str = "ObjectLiteralMethodSnippet/";
// Case completions for switch statements
pub const SOURCE_SWITCH_CASES: &str = "SwitchCases/";
// Completions for an object literal expression
pub const SOURCE_OBJECT_LITERAL_MEMBER_WITH_COMMA: &str = "ObjectLiteralMemberWithComma/";
