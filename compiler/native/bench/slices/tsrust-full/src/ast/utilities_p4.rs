//! Port of typescript-go `internal/ast/utilities.go` lines 2729-3631.

use crate::prelude::*;
use std::borrow::Cow;

// Go: ast/utilities.go:2797 IsRequireCall

// Go: ast/utilities.go:2811 IsRequireVariableStatement

// Go: ast/utilities.go:2820 GetJSXImplicitImportBase

// Go: ast/utilities.go:2843 GetJSXRuntimeImport

// Go: ast/utilities.go:2850 GetPragmaFromSourceFile
// PORT: Go `*Pragma` (nil when absent) -> `Option<FileRef<Pragma>>`, a guard
// on an entry of `source_file_info(file).pragmas`. Pass it to
// `get_pragma_argument` with `as_deref()`.

// Go: ast/utilities.go:2862 GetPragmaArgument

// Go: ast/utilities.go:2874 IsVariableDeclarationInitializedToRequire
// Of the form: `const x = require("x")` or `const { x } = require("x")` or with `var` or `let`
// The variable must not be exported and must not have a type annotation, even a jsdoc one.
// The initializer must be a call to `require` with a string literal or a string literal-like argument.

// Go: ast/utilities.go:2881 IsVariableDeclarationInitializedToBareOrAccessedRequire

// Go: ast/utilities.go:2885 isVariableDeclarationInitializedWithRequireHelper

// Go: ast/utilities.go:2905 GetModuleSpecifierOfBareOrAccessedRequire

// Go: ast/utilities.go:2918 IsModuleExportsAccessExpression

// Go: ast/utilities.go:2927 IsModuleExportsQualifiedName

// Go: ast/utilities.go:2931 IsCheckJSEnabledForFile

// Go: ast/utilities.go:2938 IsPlainJSFile

// Go: ast/utilities.go:2942 GetLeftmostAccessExpression

// Go: ast/utilities.go:2949 IsTypeOnlyImportDeclaration

// Go: ast/utilities.go:2961 isTypeOnlyExportDeclaration

// Go: ast/utilities.go:2974 IsTypeOnlyImportOrExportDeclaration

// Go: ast/utilities.go:2978 IsExclusivelyTypeOnlyImportOrExport

// Go: ast/utilities.go:2994 GetClassLikeDeclarationOfSymbol

// Go: ast/utilities.go:2998 IsCallLikeExpression

// Go: ast/utilities.go:3009 IsJsxCallLike

// Go: ast/utilities.go:3017 IsCallLikeOrFunctionLikeExpression

// Go: ast/utilities.go:3021 NodeHasKind

// Go: ast/utilities.go:3028 IsContextualKeyword
pub fn is_contextual_keyword(token: SyntaxKind) -> bool {
    // PORT: compare discriminants; crate::astdata::SyntaxKind does not derive Ord.
    SyntaxKind::FIRST_CONTEXTUAL_KEYWORD as u16 <= token as u16
        && token as u16 <= SyntaxKind::LAST_CONTEXTUAL_KEYWORD as u16
}

// Go: ast/utilities.go:3032 IsThisInTypeQuery

// Go: ast/utilities.go:3043 IsLet
// Gets whether a bound `VariableDeclaration` or `VariableDeclarationList` is part of a `let` declaration.

// Go: ast/utilities.go:3047 IsClassMemberModifier
pub fn is_class_member_modifier(token: SyntaxKind) -> bool {
    is_parameter_property_modifier(token)
        || token == SyntaxKind::StaticKeyword
        || token == SyntaxKind::OverrideKeyword
        || token == SyntaxKind::AccessorKeyword
}

// Go: ast/utilities.go:3052 IsParameterPropertyModifier
pub fn is_parameter_property_modifier(kind: SyntaxKind) -> bool {
    modifier_to_flag(kind).intersects(ModifierFlags::PARAMETER_PROPERTY_MODIFIER)
}

// Go: ast/utilities.go:3056 ForEachChildAndJSDoc
// PORT: Go `Visitor` (`func(*Node) bool`) -> `&mut dyn FnMut(Node) -> bool`.
// `visitNodes` is inlined as a loop over `node.js_doc(file)`.

// Go: ast/utilities.go:3063 HasTypeArguments

// Go: ast/utilities.go:3073 IsTypeReferenceType

// Go: ast/utilities.go:3077 IsVariableLike

// Go: ast/utilities.go:3086 HasInitializer

// Go: ast/utilities.go:3097 IsVariableParameterOrProperty

// PORT: Go `node.FunctionLikeData() != nil`. These are exactly the node kinds
// whose Go data embeds `FunctionLikeBase` (directly, through
// `FunctionLikeWithBodyBase`, `AccessorDeclarationBase` or
// `FunctionOrConstructorTypeNodeBase`). For them Go `node.Type()` returns
// `FunctionLikeData().Type`.

// Go: ast/utilities.go:3106 GetTypeAnnotationNode

// Go: ast/utilities.go:3123 IsObjectTypeDeclaration

// Go: ast/utilities.go:3127 IsClassOrTypeElement

// Go: ast/utilities.go:3131 GetClassExtendsHeritageElement

// Go: ast/utilities.go:3139 IsTypeKeywordToken

// Go: ast/utilities.go:3144 IsJSDocSingleCommentNodeList
// See `IsJSDocSingleCommentNode`.

// Go: ast/utilities.go:3156 IsJSDocSingleCommentNodeComment
// See `IsJSDocSingleCommentNode`.

// Go: ast/utilities.go:3165 IsJSDocSingleCommentNode
// In Strada, if a JSDoc node has a single comment, that comment is represented as a string property
// as a simplification, and therefore that comment is not visited by `forEachChild`.

// Go: ast/utilities.go:3169 IsValidTypeOnlyAliasUseSite

// Go: ast/utilities.go:3177 isIdentifierInNonEmittingHeritageClause

// Go: ast/utilities.go:3188 isPartOfPossiblyValidTypeOrAbstractComputedPropertyName

// Go: ast/utilities.go:3201 isShorthandPropertyNameUseSite

// Go: ast/utilities.go:3205 GetPropertyNameForPropertyNameNode

// Go: ast/utilities.go:3205 GetPropertyNameForPropertyNameNode
// Same logic, but it does not allocate. Node texts are `&'static str`, so
// only the minus-signed numeric name (`[-1]`) needs a new String. Hot
// callers (getLiteralTypeFromPropertyName) use this to skip a String per call.

// Go: ast/utilities.go:3227 IsPartOfTypeOnlyImportOrExportDeclaration

// Go: ast/utilities.go:3231 IsEmittableImport

// Go: ast/utilities.go:3243 IsResolutionModeOverrideHost

// PORT: Go reads the `Attributes` field of ImportTypeNode, ImportDeclaration
// and ExportDeclaration. In Rust `node.attributes()` is the Go `Node.Attributes()`
// method (JSX only), and fields.rs does not generate a clashing field accessor.
// The `Attributes` field is the only child of kind `ImportAttributes` on these
// kinds, so we find it with `for_each_child`.

// Go: ast/utilities.go:3254 HasResolutionModeOverride

// Go: ast/utilities.go:3274 IsStringTextContainingNode

// Go: ast/utilities.go:3278 IsTemplateLiteralKind
pub fn is_template_literal_kind(kind: SyntaxKind) -> bool {
    // PORT: compare discriminants; crate::astdata::SyntaxKind does not derive Ord.
    SyntaxKind::FIRST_TEMPLATE_TOKEN as u16 <= kind as u16
        && kind as u16 <= SyntaxKind::LAST_TEMPLATE_TOKEN as u16
}

// Go: ast/utilities.go:3282 IsTemplateLiteralToken

// Go: ast/utilities.go:3286 GetExternalModuleImportEqualsDeclarationExpression

// Go: ast/utilities.go:3291 CreateModifiersFromModifierFlags
pub fn create_modifiers_from_modifier_flags(
    flags: ModifierFlags,
    create_modifier: &mut dyn FnMut(SyntaxKind) -> Node,
) -> Vec<Node> {
    let mut result: Vec<Node> = Vec::new();
    if flags.intersects(ModifierFlags::EXPORT) {
        result.push(create_modifier(SyntaxKind::ExportKeyword));
    }
    if flags.intersects(ModifierFlags::AMBIENT) {
        result.push(create_modifier(SyntaxKind::DeclareKeyword));
    }
    if flags.intersects(ModifierFlags::DEFAULT) {
        result.push(create_modifier(SyntaxKind::DefaultKeyword));
    }
    if flags.intersects(ModifierFlags::CONST) {
        result.push(create_modifier(SyntaxKind::ConstKeyword));
    }
    if flags.intersects(ModifierFlags::PUBLIC) {
        result.push(create_modifier(SyntaxKind::PublicKeyword));
    }
    if flags.intersects(ModifierFlags::PRIVATE) {
        result.push(create_modifier(SyntaxKind::PrivateKeyword));
    }
    if flags.intersects(ModifierFlags::PROTECTED) {
        result.push(create_modifier(SyntaxKind::ProtectedKeyword));
    }
    if flags.intersects(ModifierFlags::ABSTRACT) {
        result.push(create_modifier(SyntaxKind::AbstractKeyword));
    }
    if flags.intersects(ModifierFlags::STATIC) {
        result.push(create_modifier(SyntaxKind::StaticKeyword));
    }
    if flags.intersects(ModifierFlags::OVERRIDE) {
        result.push(create_modifier(SyntaxKind::OverrideKeyword));
    }
    if flags.intersects(ModifierFlags::READONLY) {
        result.push(create_modifier(SyntaxKind::ReadonlyKeyword));
    }
    if flags.intersects(ModifierFlags::ACCESSOR) {
        result.push(create_modifier(SyntaxKind::AccessorKeyword));
    }
    if flags.intersects(ModifierFlags::ASYNC) {
        result.push(create_modifier(SyntaxKind::AsyncKeyword));
    }
    if flags.intersects(ModifierFlags::IN) {
        result.push(create_modifier(SyntaxKind::InKeyword));
    }
    if flags.intersects(ModifierFlags::OUT) {
        result.push(create_modifier(SyntaxKind::OutKeyword));
    }
    result
}

// Go: ast/utilities.go:3341 GetThisParameter

// Go: ast/utilities.go:3352 ReplaceModifiers

// Go: ast/utilities.go:3590 IsLateVisibilityPaintedStatement

// Go: ast/utilities.go:3609 IsExternalModuleAugmentation

// Go: ast/utilities.go:3613 GetSourceFileOfModule

// Go: ast/utilities.go:3621 GetNonAugmentationDeclaration

// Go: ast/utilities.go:3627 IsTypeDeclaration

// Go: ast/utilities.go:3640 IsTypeDeclarationName

// Go: ast/utilities.go:3646 IsRightSideOfPropertyAccess

// Go: ast/utilities.go:3650 IsArgumentExpressionOfElementAccess

// Go: ast/utilities.go:3654 ClimbPastPropertyAccess

// Go: ast/utilities.go:3661 climbPastPropertyOrElementAccess

// Go: ast/utilities.go:3668 selectExpressionOfCallOrNewExpressionOrDecorator

// Go: ast/utilities.go:3675 selectTagOfTaggedTemplateExpression

// Go: ast/utilities.go:3682 selectTagNameOfJsxOpeningLikeElement

// Go: ast/utilities.go:3689 IsCallExpressionTarget
