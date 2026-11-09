//! Port of `ast/utilities.go` lines 3632-4564.

use crate::prelude::*;

// Go: ast/utilities.go:3693 IsNewExpressionTarget

// Go: ast/utilities.go:3697 IsCallOrNewExpressionTarget

// Go: ast/utilities.go:3701 IsTaggedTemplateTag

// Go: ast/utilities.go:3705 IsDecoratorTarget

// Go: ast/utilities.go:3709 IsJsxOpeningLikeElementTagName

// Go: ast/utilities.go:3713 isCalleeWorker
// PORT: Go func params become plain `fn` pointers; all callers pass package functions.

// Go: ast/utilities.go:3735 IsRightSideOfQualifiedNameOrPropertyAccess

// Go: ast/utilities.go:3748 ShouldTransformImportCall

// Go: ast/utilities.go:3756 HasQuestionToken

// Go: ast/utilities.go:3760 IsJsxOpeningLikeElement

// Go: ast/utilities.go:3764 GetInvokedExpression

// Go: ast/utilities.go:3779 IsCallOrNewExpression

// Go: ast/utilities.go:3783 IndexOfNode

// Go: ast/utilities.go:3791 CompareNodePositions

// Go: ast/utilities.go:3795 IsUnterminatedLiteral

// Gets a value indicating whether a class element is either a static or an instance property declaration with an initializer.
// Go: ast/utilities.go:3801 IsInitializedProperty

// Go: ast/utilities.go:3806 IsTrivia
pub fn is_trivia(token: SyntaxKind) -> bool {
    (SyntaxKind::FIRST_TRIVIA_TOKEN as u16) <= (token as u16)
        && (token as u16) <= (SyntaxKind::LAST_TRIVIA_TOKEN as u16)
}

// Go: ast/utilities.go:3810 HasDecorators

/// Go `hasFileNameImpl` (implements the Go `HasFileName` interface).
/// PORT: `tspath.Path` is a Go string type, ported as `String`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HasFileNameImpl {
    pub file_name: String,
    pub path: String,
}

// Go: ast/utilities.go:3819 NewHasFileName
pub fn new_has_file_name(file_name: &str, path: &str) -> HasFileNameImpl {
    HasFileNameImpl {
        file_name: file_name.to_string(),
        path: path.to_string(),
    }
}

impl HasFileNameImpl {
    // Go: ast/utilities.go:3765 (*hasFileNameImpl).FileName
    pub fn file_name(&self) -> String {
        self.file_name.clone()
    }

    // Go: ast/utilities.go:3769 (*hasFileNameImpl).Path
    pub fn path(&self) -> String {
        self.path.clone()
    }
}

// Go: ast/utilities.go:3834 GetSemanticJsxChildren

// Returns true if the node kind has a comment property.
// Go: ast/utilities.go:3848 hasComment
pub fn has_comment(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::JsDoc
            | SyntaxKind::JsDocUnknownTag
            | SyntaxKind::JsDocAugmentsTag
            | SyntaxKind::JsDocImplementsTag
            | SyntaxKind::JsDocDeprecatedTag
            | SyntaxKind::JsDocPublicTag
            | SyntaxKind::JsDocPrivateTag
            | SyntaxKind::JsDocProtectedTag
            | SyntaxKind::JsDocReadonlyTag
            | SyntaxKind::JsDocOverrideTag
            | SyntaxKind::JsDocCallbackTag
            | SyntaxKind::JsDocOverloadTag
            | SyntaxKind::JsDocParameterTag
            | SyntaxKind::JsDocPropertyTag
            | SyntaxKind::JsDocReturnTag
            | SyntaxKind::JsDocThisTag
            | SyntaxKind::JsDocTypeTag
            | SyntaxKind::JsDocTemplateTag
            | SyntaxKind::JsDocTypedefTag
            | SyntaxKind::JsDocSeeTag
            | SyntaxKind::JsDocThrowsTag
            | SyntaxKind::JsDocSatisfiesTag
            | SyntaxKind::JsDocImportTag
    )
}

// Go: ast/utilities.go:3862 IsAssignmentPattern

// Go: ast/utilities.go:3866 GetElementsOfBindingOrAssignmentPattern

// Go: ast/utilities.go:3879 IsDeclarationBindingElement


// Go: ast/utilities.go:3961 TryGetPropertyNameOfBindingOrAssignmentElement


// Go: ast/utilities.go:4038 IsEmptyObjectLiteral

// Go: ast/utilities.go:4042 IsEmptyArrayLiteral

// Go: ast/utilities.go:4046 GetRestIndicatorOfBindingOrAssignmentElement

// Go: ast/utilities.go:4058 IsJSDocNameReferenceContext

// GetJSDocRoot returns the containing JSDoc node for a node inside a JSDoc comment.
// Go: ast/utilities.go:4065 GetJSDocRoot

// GetJSDocHost returns the declaration that the JSDoc comment containing the given node is attached to.
// Go: ast/utilities.go:4072 GetJSDocHost

// GetHostSignatureFromJSDoc returns the function-like declaration that hosts the JSDoc comment
// containing the given node. This is used to resolve @link references to parameters.
// Go: ast/utilities.go:4082 GetHostSignatureFromJSDoc

// Finds the declaration that owns the JSDoc for a function-like node.
// Keep these hosts aligned with JSDoc parameter reparsing so unmatched @param diagnostics use the same attachment rules.
// Keep in sync with getNextJSDocCommentLocation in the API's src/ast/jsdoc.ts
// Go: ast/utilities.go:4100 GetNextJSDocCommentLocation

// Go: ast/utilities.go:4115 IsImportOrImportEqualsDeclaration

// Go: ast/utilities.go:4119 IsPrimitiveLiteralValue

// Go: ast/utilities.go:4142 HasInferredType

// Go: ast/utilities.go:4166 IsKeyword
pub fn is_keyword(token: SyntaxKind) -> bool {
    (SyntaxKind::FIRST_KEYWORD as u16) <= (token as u16)
        && (token as u16) <= (SyntaxKind::LAST_KEYWORD as u16)
}

// Go: ast/utilities.go:4170 IsNonContextualKeyword
pub fn is_non_contextual_keyword(token: SyntaxKind) -> bool {
    is_keyword(token) && !is_contextual_keyword(token)
}

// Go: ast/utilities.go:4174 HasModifier

// Go: ast/utilities.go:4178 IsExpandoInitializer

// Go: ast/utilities.go:4191 GetContainingFunction

// Go: ast/utilities.go:4195 ImportFromModuleSpecifier

// Go: ast/utilities.go:4203 TryGetImportFromModuleSpecifier

// Go: ast/utilities.go:4226 IsImplicitlyExportedJSDocDeclaration

// Go: ast/utilities.go:4238 HasContextSensitiveParameters

// Go: ast/utilities.go:4257 IsInfinityOrNaNString
pub fn is_infinity_or_na_n_string(name: &str) -> bool {
    name == "Infinity" || name == "-Infinity" || name == "NaN"
}

// Go: ast/utilities.go:4261 GetFirstConstructorWithBody

// Returns true for nodes that are considered executable for the purposes of unreachable code detection.
// Go: ast/utilities.go:4271 IsPotentiallyExecutableNode

// Go: ast/utilities.go:4288 HasAbstractModifier

// Go: ast/utilities.go:4292 HasAmbientModifier

// Go: ast/utilities.go:4296 NodeCanBeDecorated

// Go: ast/utilities.go:4330 ClassOrConstructorParameterIsDecorated

// Go: ast/utilities.go:4338 ClassElementOrClassElementParameterIsDecorated

// Go: ast/utilities.go:4373 NodeIsDecorated

// Go: ast/utilities.go:4377 NodeOrChildIsDecorated

// Go: ast/utilities.go:4381 ChildIsDecorated

/// Go `ast.AllAccessorDeclarations`. Typed Go pointers (`*AccessorDeclaration`,
/// `*SetAccessorDeclaration`, `*GetAccessorDeclaration`) are all `Node`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AllAccessorDeclarations {
    pub first_accessor: Node,
    pub second_accessor: Node,
    pub set_accessor: Node,
    pub get_accessor: Node,
}

// Go: ast/utilities.go:4405 GetAllAccessorDeclarationsForDeclaration

// Go: ast/utilities.go:4455 GetAllAccessorDeclarations

// Go: ast/utilities.go:4476 IsAsyncFunction


// Go: ast/utilities.go:4506 TagNamesAreEquivalent

// Go: ast/utilities.go:4525 IsTagName

// We want to store any numbers/strings if they were a name that could be
// related to a declaration.  So, if we have 'import x = require("something")'
// then we want 'something' to be in the name table.  Similarly, if we have
// "a['propname']" then we want to store "propname" in the name table.
// Go: ast/utilities.go:4533 literalIsName

// Go: ast/utilities.go:4540 isArgumentOfElementAccessExpression

// If the given node is part of a subtree of JSDoc nodes that have been cloned into a reparsed construct,
// return the corresponding reparsed clone in the subtree. Otherwise, just return the node.
// Go: ast/utilities.go:4548 GetReparsedNodeForNode

// PORT: Go `core.TextRange.ContainedBy` (`t2.pos <= t.pos && t2.end >= t.end`), inlined
// here so this file does not depend on a TextRange method name.
fn text_range_contained_by(t: TextRange, t2: TextRange) -> bool {
    t2.pos() <= t.pos() && t2.end() >= t.end()
}

// Go: ast/utilities.go:4566 findCloneInNode

// Go: ast/utilities.go:4584 IsExpandoPropertyDeclaration

// IsSuperProperty checks if a node is super.x or super[x].
// Go: ast/utilities.go:4589 IsSuperProperty

// Indicates whether a node is a potential source of an assigned name for a class, function, or arrow function.
// Go: ast/utilities.go:4595 IsNamedEvaluationSource

// Indicates whether a property name is the special `__proto__` property.
// Per the ECMA-262 spec, this only matters for property assignments whose name is
// the Identifier `__proto__`, or the string literal `"__proto__"`, but not for
// computed property names.
// Go: ast/utilities.go:4624 IsProtoSetter

// Go: ast/utilities.go:4628 IsStringLiteralLikeType (ts#63931)
