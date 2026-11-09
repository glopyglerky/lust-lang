//! Port of the Go `NodeFactory.Update*` methods (`ast/ast_generated.go`),
//! `UpdateSourceFile` (`ast/ast.go`), `NodeFactoryHooks` and `updateNode`
//! (`ast/ast.go`).
//!
//! Each `update_*` method returns `node` itself when every argument equals the
//! current child (Go pointer equality on `Node`, `NodeList` and
//! `ModifierList` handles). Otherwise it creates a new node with the Go `New*`
//! constructor and copies `Flags` and `Loc` from `node` (`update_node`).

use crate::prelude::*;

// Go: ast/ast.go:61 NodeFactoryHooks
/// Go `ast.NodeFactoryHooks`. The printer's `EmitContext` sets them.
#[derive(Clone, Default)]
pub struct NodeFactoryHooks {
    /// Hooks the creation of a node.
    pub on_create: Option<Rc<dyn Fn(Node)>>,
    /// Hooks the updating of a node: `(updated, original)`.
    pub on_update: Option<Rc<dyn Fn(Node, Node)>>,
    /// Hooks the cloning of a node: `(updated, original)`.
    pub on_clone: Option<Rc<dyn Fn(Node, Node)>>,
}

impl std::fmt::Debug for NodeFactoryHooks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeFactoryHooks")
            .field("on_create", &self.on_create.is_some())
            .field("on_update", &self.on_update.is_some())
            .field("on_clone", &self.on_clone.is_some())
            .finish()
    }
}

// Go: ast/ast.go:103 updateNode

impl NodeFactory {
    // Go: ast/ast_generated.go:685 UpdateQualifiedName

    // Go: ast/ast_generated.go:729 UpdateComputedPropertyName

    // Go: ast/ast_generated.go:772 UpdateDecorator

    // Go: ast/ast_generated.go:836 UpdateIfStatement

    // Go: ast/ast_generated.go:882 UpdateDoStatement

    // Go: ast/ast_generated.go:927 UpdateWhileStatement

    // Go: ast/ast_generated.go:977 UpdateForStatement

    // Go: ast/ast_generated.go:1033 UpdateForInOrOfStatement

    // Go: ast/ast_generated.go:1078 UpdateBreakStatement

    // Go: ast/ast_generated.go:1116 UpdateContinueStatement

    // Go: ast/ast_generated.go:1155 UpdateReturnStatement

    // Go: ast/ast_generated.go:1196 UpdateWithStatement

    // Go: ast/ast_generated.go:1242 UpdateSwitchStatement

    // Go: ast/ast_generated.go:1287 UpdateCaseBlock

    // Go: ast/ast_generated.go:1333 UpdateCaseOrDefaultClause

    // Go: ast/ast_generated.go:1381 UpdateThrowStatement

    // Go: ast/ast_generated.go:1428 UpdateTryStatement

    // Go: ast/ast_generated.go:1476 UpdateCatchClause

    // Go: ast/ast_generated.go:1537 UpdateLabeledStatement

    // Go: ast/ast_generated.go:1580 UpdateExpressionStatement

    // Go: ast/ast_generated.go:1626 UpdateBlock

    // Go: ast/ast_generated.go:1671 UpdateVariableStatement

    // Go: ast/ast_generated.go:1718 UpdateVariableDeclaration

    // Go: ast/ast_generated.go:1766 UpdateVariableDeclarationList

    // Go: ast/ast_generated.go:1805 UpdateBindingPattern

    // Go: ast/ast_generated.go:1859 UpdateParameterDeclaration

    // Go: ast/ast_generated.go:1916 UpdateBindingElement

    // Go: ast/ast_generated.go:1962 UpdateMissingDeclaration

    // Go: ast/ast_generated.go:2013 UpdateFunctionDeclaration

    // Go: ast/ast_generated.go:2067 UpdateClassDeclaration

    // Go: ast/ast_generated.go:2118 UpdateClassExpression

    // Go: ast/ast_generated.go:2167 UpdateHeritageClause

    // Go: ast/ast_generated.go:2216 UpdateInterfaceDeclaration

    // Go: ast/ast_generated.go:2281 UpdateTypeAliasDeclaration

    // Go: ast/ast_generated.go:2348 UpdateEnumMember

    // Go: ast/ast_generated.go:2397 UpdateEnumDeclaration

    // Go: ast/ast_generated.go:2440 UpdateModuleBlock

    // Go: ast/ast_generated.go:2543 UpdateImportDeclaration

    // Go: ast/ast_generated.go:2609 UpdateExternalModuleReference

    // Go: ast/ast_generated.go:2653 UpdateNamespaceImport

    // Go: ast/ast_generated.go:2700 UpdateNamedImports

    // Go: ast/ast_generated.go:2750 UpdateExportAssignment

    // Go: ast/ast_generated.go:2792 UpdateNamespaceExportDeclaration

    // Go: ast/ast_generated.go:2835 UpdateNamespaceExport

    // Go: ast/ast_generated.go:2882 UpdateNamedExports

    // Go: ast/ast_generated.go:2931 UpdateExportSpecifier

    // Go: ast/ast_generated.go:2978 UpdateCallSignatureDeclaration

    // Go: ast/ast_generated.go:3021 UpdateConstructSignatureDeclaration

    // Go: ast/ast_generated.go:3069 UpdateConstructorDeclaration

    // Go: ast/ast_generated.go:3117 UpdateGetAccessorDeclaration

    // Go: ast/ast_generated.go:3170 UpdateSetAccessorDeclaration

    // Go: ast/ast_generated.go:3225 UpdateIndexSignatureDeclaration

    // Go: ast/ast_generated.go:3272 UpdateMethodSignatureDeclaration

    // Go: ast/ast_generated.go:3333 UpdateMethodDeclaration

    // Go: ast/ast_generated.go:3392 UpdatePropertySignatureDeclaration

    // Go: ast/ast_generated.go:3447 UpdatePropertyDeclaration

    // Go: ast/ast_generated.go:3523 UpdateClassStaticBlockDeclaration

    // Go: ast/ast_generated.go:3740 UpdateBinaryExpression

    // Go: ast/ast_generated.go:3784 UpdatePrefixUnaryExpression

    // Go: ast/ast_generated.go:3828 UpdatePostfixUnaryExpression

    // Go: ast/ast_generated.go:3872 UpdateYieldExpression

    // Go: ast/ast_generated.go:3921 UpdateArrowFunction

    // Go: ast/ast_generated.go:3978 UpdateFunctionExpression

    // Go: ast/ast_generated.go:4029 UpdateAsExpression

    // Go: ast/ast_generated.go:4069 UpdateSatisfiesExpression

    // Go: ast/ast_generated.go:4116 UpdateConditionalExpression

    // Go: ast/ast_generated.go:4174 UpdatePropertyAccessExpression

    // Go: ast/ast_generated.go:4224 UpdateElementAccessExpression

    // Go: ast/ast_generated.go:4278 UpdateCallExpression

    // Go: ast/ast_generated.go:4324 UpdateNewExpression

    // Go: ast/ast_generated.go:4366 UpdateMetaProperty

    // Go: ast/ast_generated.go:4410 UpdateNonNullExpression

    // Go: ast/ast_generated.go:4448 UpdateSpreadElement

    // Go: ast/ast_generated.go:4489 UpdateTemplateExpression

    // Go: ast/ast_generated.go:4534 UpdateTemplateSpan

    // Go: ast/ast_generated.go:4586 UpdateTaggedTemplateExpression

    // Go: ast/ast_generated.go:4627 UpdateParenthesizedExpression

    // Go: ast/ast_generated.go:4672 UpdateArrayLiteralExpression

    // Go: ast/ast_generated.go:4718 UpdateObjectLiteralExpression

    // Go: ast/ast_generated.go:4762 UpdateSpreadAssignment

    // Go: ast/ast_generated.go:4809 UpdatePropertyAssignment

    // Go: ast/ast_generated.go:4866 UpdateShorthandPropertyAssignment

    // Go: ast/ast_generated.go:4913 UpdateDeleteExpression

    // Go: ast/ast_generated.go:4955 UpdateTypeOfExpression

    // Go: ast/ast_generated.go:4997 UpdateVoidExpression

    // Go: ast/ast_generated.go:5039 UpdateAwaitExpression

    // Go: ast/ast_generated.go:5079 UpdateTypeAssertion

    // Go: ast/ast_generated.go:5141 UpdateUnionTypeNode

    // Go: ast/ast_generated.go:5178 UpdateIntersectionTypeNode

    // Go: ast/ast_generated.go:5223 UpdateConditionalTypeNode

    // Go: ast/ast_generated.go:5266 UpdateTypeOperatorNode

    // Go: ast/ast_generated.go:5304 UpdateInferTypeNode

    // Go: ast/ast_generated.go:5342 UpdateArrayTypeNode

    // Go: ast/ast_generated.go:5382 UpdateIndexedAccessTypeNode

    // Go: ast/ast_generated.go:5421 UpdateTypeReferenceNode

    // Go: ast/ast_generated.go:5462 UpdateExpressionWithTypeArguments

    // Go: ast/ast_generated.go:5500 UpdateLiteralTypeNode

    // Go: ast/ast_generated.go:5563 UpdateTypePredicateNode

    // Go: ast/ast_generated.go:5604 UpdateImportAttribute

    // Go: ast/ast_generated.go:5656 UpdateImportAttributes

    // Go: ast/ast_generated.go:5699 UpdateTypeQueryNode

    // Go: ast/ast_generated.go:5749 UpdateMappedTypeNode

    // Go: ast/ast_generated.go:5793 UpdateTypeLiteralNode

    // Go: ast/ast_generated.go:5831 UpdateTupleTypeNode

    // Go: ast/ast_generated.go:5876 UpdateNamedTupleMember

    // Go: ast/ast_generated.go:5921 UpdateOptionalTypeNode

    // Go: ast/ast_generated.go:5959 UpdateRestTypeNode

    // Go: ast/ast_generated.go:5997 UpdateParenthesizedTypeNode

    // Go: ast/ast_generated.go:6036 UpdateFunctionTypeNode

    // Go: ast/ast_generated.go:6076 UpdateConstructorTypeNode

    // Go: ast/ast_generated.go:6197 UpdateTemplateLiteralTypeNode

    // Go: ast/ast_generated.go:6237 UpdateTemplateLiteralTypeSpan

    // Go: ast/ast_generated.go:6279 UpdateSyntheticExpression

    // Go: ast/ast_generated.go:6317 UpdatePartiallyEmittedExpression

    // Go: ast/ast_generated.go:6364 UpdateJsxElement

    // Go: ast/ast_generated.go:6404 UpdateJsxAttributes

    // Go: ast/ast_generated.go:6445 UpdateJsxNamespacedName

    // Go: ast/ast_generated.go:6492 UpdateJsxOpeningElement

    // Go: ast/ast_generated.go:6535 UpdateJsxSelfClosingElement

    // Go: ast/ast_generated.go:6578 UpdateJsxFragment

    // Go: ast/ast_generated.go:6662 UpdateJsxAttribute

    // Go: ast/ast_generated.go:6706 UpdateJsxSpreadAttribute

    // Go: ast/ast_generated.go:6744 UpdateJsxClosingElement

    // Go: ast/ast_generated.go:6784 UpdateJsxExpression

    // Go: ast/ast_generated.go:6848 UpdateSyntaxList
    // PORT: Go `core.Same` (slice identity) becomes element equality. The visitor passes the
    // unchanged children when `core.SameMap` would return the original slice.

    // Go: ast/ast_generated.go:6889 UpdateJSDoc

    // Go: ast/ast_generated.go:6927 UpdateJSDocTypeExpression

    // Go: ast/ast_generated.go:6965 UpdateJSDocNonNullableType

    // Go: ast/ast_generated.go:7003 UpdateJSDocNullableType

    // Go: ast/ast_generated.go:7062 UpdateJSDocVariadicType

    // Go: ast/ast_generated.go:7100 UpdateJSDocOptionalType

    // Go: ast/ast_generated.go:7140 UpdateJSDocTypeTag

    // Go: ast/ast_generated.go:7178 UpdateJSDocUnknownTag

    // Go: ast/ast_generated.go:7220 UpdateJSDocTemplateTag

    // Go: ast/ast_generated.go:7263 UpdateJSDocReturnTag

    // Go: ast/ast_generated.go:7301 UpdateJSDocPublicTag

    // Go: ast/ast_generated.go:7339 UpdateJSDocPrivateTag

    // Go: ast/ast_generated.go:7377 UpdateJSDocProtectedTag

    // Go: ast/ast_generated.go:7415 UpdateJSDocReadonlyTag

    // Go: ast/ast_generated.go:7453 UpdateJSDocOverrideTag

    // Go: ast/ast_generated.go:7491 UpdateJSDocDeprecatedTag

    // Go: ast/ast_generated.go:7531 UpdateJSDocSeeTag

    // Go: ast/ast_generated.go:7571 UpdateJSDocImplementsTag

    // Go: ast/ast_generated.go:7611 UpdateJSDocAugmentsTag

    // Go: ast/ast_generated.go:7651 UpdateJSDocSatisfiesTag

    // Go: ast/ast_generated.go:7691 UpdateJSDocThrowsTag

    // Go: ast/ast_generated.go:7731 UpdateJSDocThisTag

    // Go: ast/ast_generated.go:7775 UpdateJSDocImportTag

    // Go: ast/ast_generated.go:7821 UpdateJSDocCallbackTag

    // Go: ast/ast_generated.go:7868 UpdateJSDocOverloadTag

    // Go: ast/ast_generated.go:7910 UpdateJSDocTypedefTag

    // Go: ast/ast_generated.go:7958 UpdateJSDocSignature

    // Go: ast/ast_generated.go:7996 UpdateJSDocNameReference

    // Go: ast/ast_generated.go:8059 UpdateModuleDeclaration

    // Go: ast/ast_generated.go:8113 UpdateImportEqualsDeclaration

    // Go: ast/ast_generated.go:8165 UpdateExportDeclaration

    // Go: ast/ast_generated.go:8213 UpdateImportTypeNode

    // Go: ast/ast_generated.go:8261 UpdateImportClause

    // Go: ast/ast_generated.go:8310 UpdateImportSpecifier

    // Go: ast/ast_generated.go:8377 UpdateJSDocLink
    // PORT: Go `text []string` and `core.Same`; astdata stores the link text as one string, compared by value.

    // Go: ast/ast_generated.go:8421 UpdateJSDocLinkPlain
    // PORT: Go `text []string` and `core.Same`; astdata stores the link text as one string, compared by value.

    // Go: ast/ast_generated.go:8465 UpdateJSDocLinkCode
    // PORT: Go `text []string` and `core.Same`; astdata stores the link text as one string, compared by value.

    // Go: ast/ast_generated.go:8517 UpdateTypeParameterDeclaration

    // Go: ast/ast_generated.go:8565 UpdateSyntheticReferenceExpression

    // Go: ast/ast_generated.go:8611 UpdateJSDocTypeLiteral
    // PORT: Go `core.Same` (slice identity) becomes element equality. The visitor passes the
    // unchanged children when `core.SameMap` would return the original slice.

    // Go: ast/ast_generated.go:8658 UpdateJSDocParameterOrPropertyTag

    // Go: ast/ast.go:2825 UpdateSourceFile
}

