//! Port of Go `ast.NodeVisitor` (`ast/visitor.go`) and the `VisitEachChild`
//! methods (`ast/ast_generated.go`, plus `SourceFile` and
//! `visitEachChild_JSDocParameterOrPropertyTag` from `ast/ast.go`).
//!
//! PORT: Go visitor callbacks are closures that capture the visitor and any
//! state they need. Here:
//! - `NodeVisitor` owns a context value `ctx: C` (for example
//!   `&mut Checker`), so a callback can reach mutable state through the
//!   visitor it gets. `C` defaults to `()`.
//! - The `visit` callback and every hook get `(node, &mut NodeVisitor)`. Go
//!   `Visit(node)` gets only the node; the visitor argument replaces the Go
//!   closure capture of the visitor.
//! - Callbacks are `Rc<dyn Fn>`, so a callback can call back into the visitor
//!   (Go recursion through `v.VisitEachChild`). Keep callback state in
//!   `Cell`/`RefCell` or in `ctx`.
//! - Go has exported and unexported methods with the same name (`VisitNode`
//!   and `visitNode`). The exported ones keep the plain snake names
//!   (`visit_node`, `visit_nodes`, `visit_modifiers`,
//!   `visit_embedded_statement`), as the printer plan's contract gives them.
//!   The unexported hook-aware ones get a `_hooked` suffix.

use crate::astdata::NodeData;
use crate::prelude::*;


// Go: ast/visitor.go:16 NodeVisitorHooks



// Go: ast/visitor.go:9 NodeVisitor


// Go: ast/visitor.go:28 NewNodeVisitor
// PORT: Go takes a nilable func; set `visit` to `None` after creation for a
// nil callback. `ctx` is the callback state (see the module comment).





impl Node {
    // Go: ast/ast.go:197 (n *Node) VisitEachChild

}

// Go: ast/ast.go:2653 (node *SourceFile) VisitEachChild

// Go: ast/ast.go:3179 visitEachChild_JSDocParameterOrPropertyTag

// Go: ast/ast_generated.go:861 (node *QualifiedName) VisitEachChild

// Go: ast/ast_generated.go:905 (node *ComputedPropertyName) VisitEachChild

// Go: ast/ast_generated.go:948 (node *Decorator) VisitEachChild

// Go: ast/ast_generated.go:1012 (node *IfStatement) VisitEachChild

// Go: ast/ast_generated.go:1058 (node *DoStatement) VisitEachChild

// Go: ast/ast_generated.go:1103 (node *WhileStatement) VisitEachChild

// Go: ast/ast_generated.go:1156 (node *ForStatement) VisitEachChild

// Go: ast/ast_generated.go:1212 (node *ForInOrOfStatement) VisitEachChild

// Go: ast/ast_generated.go:1254 (node *BreakStatement) VisitEachChild

// Go: ast/ast_generated.go:1292 (node *ContinueStatement) VisitEachChild

// Go: ast/ast_generated.go:1331 (node *ReturnStatement) VisitEachChild

// Go: ast/ast_generated.go:1372 (node *WithStatement) VisitEachChild

// Go: ast/ast_generated.go:1418 (node *SwitchStatement) VisitEachChild

// Go: ast/ast_generated.go:1463 (node *CaseBlock) VisitEachChild

// Go: ast/ast_generated.go:1509 (node *CaseOrDefaultClause) VisitEachChild

// Go: ast/ast_generated.go:1557 (node *ThrowStatement) VisitEachChild

// Go: ast/ast_generated.go:1604 (node *TryStatement) VisitEachChild

// Go: ast/ast_generated.go:1652 (node *CatchClause) VisitEachChild

// Go: ast/ast_generated.go:1713 (node *LabeledStatement) VisitEachChild

// Go: ast/ast_generated.go:1756 (node *ExpressionStatement) VisitEachChild

// Go: ast/ast_generated.go:1802 (node *Block) VisitEachChild

// Go: ast/ast_generated.go:1847 (node *VariableStatement) VisitEachChild

// Go: ast/ast_generated.go:1897 (node *VariableDeclaration) VisitEachChild

// Go: ast/ast_generated.go:1942 (node *VariableDeclarationList) VisitEachChild

// Go: ast/ast_generated.go:1981 (node *BindingPattern) VisitEachChild

// Go: ast/ast_generated.go:2040 (node *ParameterDeclaration) VisitEachChild

// Go: ast/ast_generated.go:2095 (node *BindingElement) VisitEachChild

// Go: ast/ast_generated.go:2138 (node *MissingDeclaration) VisitEachChild

// Go: ast/ast_generated.go:2196 (node *FunctionDeclaration) VisitEachChild

// Go: ast/ast_generated.go:2247 (node *ClassDeclaration) VisitEachChild

// Go: ast/ast_generated.go:2297 (node *ClassExpression) VisitEachChild

// Go: ast/ast_generated.go:2342 (node *HeritageClause) VisitEachChild

// Go: ast/ast_generated.go:2395 (node *InterfaceDeclaration) VisitEachChild

// Go: ast/ast_generated.go:2466 (node *TypeAliasDeclaration) VisitEachChild

// Go: ast/ast_generated.go:2522 (node *EnumMember) VisitEachChild

// Go: ast/ast_generated.go:2571 (node *EnumDeclaration) VisitEachChild

// Go: ast/ast_generated.go:2614 (node *ModuleBlock) VisitEachChild

// Go: ast/ast_generated.go:2726 (node *ImportDeclaration) VisitEachChild

// Go: ast/ast_generated.go:2782 (node *ExternalModuleReference) VisitEachChild

// Go: ast/ast_generated.go:2826 (node *NamespaceImport) VisitEachChild

// Go: ast/ast_generated.go:2873 (node *NamedImports) VisitEachChild

// Go: ast/ast_generated.go:2923 (node *ExportAssignment) VisitEachChild

// Go: ast/ast_generated.go:2965 (node *NamespaceExportDeclaration) VisitEachChild

// Go: ast/ast_generated.go:3008 (node *NamespaceExport) VisitEachChild

// Go: ast/ast_generated.go:3055 (node *NamedExports) VisitEachChild

// Go: ast/ast_generated.go:3104 (node *ExportSpecifier) VisitEachChild

// Go: ast/ast_generated.go:3151 (node *CallSignatureDeclaration) VisitEachChild

// Go: ast/ast_generated.go:3194 (node *ConstructSignatureDeclaration) VisitEachChild

// Go: ast/ast_generated.go:3247 (node *ConstructorDeclaration) VisitEachChild

// Go: ast/ast_generated.go:3296 (node *GetAccessorDeclaration) VisitEachChild

// Go: ast/ast_generated.go:3349 (node *SetAccessorDeclaration) VisitEachChild

// Go: ast/ast_generated.go:3398 (node *IndexSignatureDeclaration) VisitEachChild

// Go: ast/ast_generated.go:3449 (node *MethodSignatureDeclaration) VisitEachChild

// Go: ast/ast_generated.go:3512 (node *MethodDeclaration) VisitEachChild

// Go: ast/ast_generated.go:3566 (node *PropertySignatureDeclaration) VisitEachChild

// Go: ast/ast_generated.go:3620 (node *PropertyDeclaration) VisitEachChild

// Go: ast/ast_generated.go:3692 (node *ClassStaticBlockDeclaration) VisitEachChild

// Go: ast/ast_generated.go:3918 (node *BinaryExpression) VisitEachChild

// Go: ast/ast_generated.go:3958 (node *PrefixUnaryExpression) VisitEachChild

// Go: ast/ast_generated.go:4002 (node *PostfixUnaryExpression) VisitEachChild

// Go: ast/ast_generated.go:4046 (node *YieldExpression) VisitEachChild

// Go: ast/ast_generated.go:4101 (node *ArrowFunction) VisitEachChild

// Go: ast/ast_generated.go:4159 (node *FunctionExpression) VisitEachChild

// Go: ast/ast_generated.go:4203 (node *AsExpression) VisitEachChild

// Go: ast/ast_generated.go:4243 (node *SatisfiesExpression) VisitEachChild

// Go: ast/ast_generated.go:4294 (node *ConditionalExpression) VisitEachChild

// Go: ast/ast_generated.go:4348 (node *PropertyAccessExpression) VisitEachChild

// Go: ast/ast_generated.go:4398 (node *ElementAccessExpression) VisitEachChild

// Go: ast/ast_generated.go:4455 (node *CallExpression) VisitEachChild

// Go: ast/ast_generated.go:4498 (node *NewExpression) VisitEachChild

// Go: ast/ast_generated.go:4540 (node *MetaProperty) VisitEachChild

// Go: ast/ast_generated.go:4584 (node *NonNullExpression) VisitEachChild

// Go: ast/ast_generated.go:4622 (node *SpreadElement) VisitEachChild

// Go: ast/ast_generated.go:4663 (node *TemplateExpression) VisitEachChild

// Go: ast/ast_generated.go:4708 (node *TemplateSpan) VisitEachChild

// Go: ast/ast_generated.go:4763 (node *TaggedTemplateExpression) VisitEachChild

// Go: ast/ast_generated.go:4801 (node *ParenthesizedExpression) VisitEachChild

// Go: ast/ast_generated.go:4846 (node *ArrayLiteralExpression) VisitEachChild

// Go: ast/ast_generated.go:4892 (node *ObjectLiteralExpression) VisitEachChild

// Go: ast/ast_generated.go:4936 (node *SpreadAssignment) VisitEachChild

// Go: ast/ast_generated.go:4986 (node *PropertyAssignment) VisitEachChild

// Go: ast/ast_generated.go:5043 (node *ShorthandPropertyAssignment) VisitEachChild

// Go: ast/ast_generated.go:5085 (node *DeleteExpression) VisitEachChild

// Go: ast/ast_generated.go:5127 (node *TypeOfExpression) VisitEachChild

// Go: ast/ast_generated.go:5169 (node *VoidExpression) VisitEachChild

// Go: ast/ast_generated.go:5211 (node *AwaitExpression) VisitEachChild

// Go: ast/ast_generated.go:5251 (node *TypeAssertion) VisitEachChild

// Go: ast/ast_generated.go:5325 (node *UnionTypeNode) VisitEachChild

// Go: ast/ast_generated.go:5363 (node *IntersectionTypeNode) VisitEachChild

// Go: ast/ast_generated.go:5411 (node *ConditionalTypeNode) VisitEachChild

// Go: ast/ast_generated.go:5451 (node *TypeOperatorNode) VisitEachChild

// Go: ast/ast_generated.go:5489 (node *InferTypeNode) VisitEachChild

// Go: ast/ast_generated.go:5527 (node *ArrayTypeNode) VisitEachChild

// Go: ast/ast_generated.go:5567 (node *IndexedAccessTypeNode) VisitEachChild

// Go: ast/ast_generated.go:5606 (node *TypeReferenceNode) VisitEachChild

// Go: ast/ast_generated.go:5647 (node *ExpressionWithTypeArguments) VisitEachChild

// Go: ast/ast_generated.go:5685 (node *LiteralTypeNode) VisitEachChild

// Go: ast/ast_generated.go:5748 (node *TypePredicateNode) VisitEachChild

// Go: ast/ast_generated.go:5789 (node *ImportAttribute) VisitEachChild

// Go: ast/ast_generated.go:5841 (node *ImportAttributes) VisitEachChild

// Go: ast/ast_generated.go:5884 (node *TypeQueryNode) VisitEachChild

// Go: ast/ast_generated.go:5939 (node *MappedTypeNode) VisitEachChild

// Go: ast/ast_generated.go:5978 (node *TypeLiteralNode) VisitEachChild

// Go: ast/ast_generated.go:6016 (node *TupleTypeNode) VisitEachChild

// Go: ast/ast_generated.go:6064 (node *NamedTupleMember) VisitEachChild

// Go: ast/ast_generated.go:6106 (node *OptionalTypeNode) VisitEachChild

// Go: ast/ast_generated.go:6144 (node *RestTypeNode) VisitEachChild

// Go: ast/ast_generated.go:6182 (node *ParenthesizedTypeNode) VisitEachChild

// Go: ast/ast_generated.go:6222 (node *FunctionTypeNode) VisitEachChild

// Go: ast/ast_generated.go:6266 (node *ConstructorTypeNode) VisitEachChild

// Go: ast/ast_generated.go:6384 (node *TemplateLiteralTypeNode) VisitEachChild

// Go: ast/ast_generated.go:6424 (node *TemplateLiteralTypeSpan) VisitEachChild

// Go: ast/ast_generated.go:6466 (node *SyntheticExpression) VisitEachChild

// Go: ast/ast_generated.go:6504 (node *PartiallyEmittedExpression) VisitEachChild

// Go: ast/ast_generated.go:6551 (node *JsxElement) VisitEachChild

// Go: ast/ast_generated.go:6591 (node *JsxAttributes) VisitEachChild

// Go: ast/ast_generated.go:6632 (node *JsxNamespacedName) VisitEachChild

// Go: ast/ast_generated.go:6679 (node *JsxOpeningElement) VisitEachChild

// Go: ast/ast_generated.go:6722 (node *JsxSelfClosingElement) VisitEachChild

// Go: ast/ast_generated.go:6765 (node *JsxFragment) VisitEachChild

// Go: ast/ast_generated.go:6849 (node *JsxAttribute) VisitEachChild

// Go: ast/ast_generated.go:6892 (node *JsxSpreadAttribute) VisitEachChild

// Go: ast/ast_generated.go:6930 (node *JsxClosingElement) VisitEachChild

// Go: ast/ast_generated.go:6970 (node *JsxExpression) VisitEachChild

// Go: ast/ast_generated.go:7034 (node *SyntaxList) VisitEachChild

// Go: ast/ast_generated.go:7075 (node *JSDoc) VisitEachChild

// Go: ast/ast_generated.go:7113 (node *JSDocTypeExpression) VisitEachChild

// Go: ast/ast_generated.go:7151 (node *JSDocNonNullableType) VisitEachChild

// Go: ast/ast_generated.go:7189 (node *JSDocNullableType) VisitEachChild

// Go: ast/ast_generated.go:7248 (node *JSDocVariadicType) VisitEachChild

// Go: ast/ast_generated.go:7286 (node *JSDocOptionalType) VisitEachChild

// Go: ast/ast_generated.go:7326 (node *JSDocTypeTag) VisitEachChild

// Go: ast/ast_generated.go:7364 (node *JSDocUnknownTag) VisitEachChild

// Go: ast/ast_generated.go:7409 (node *JSDocTemplateTag) VisitEachChild

// Go: ast/ast_generated.go:7449 (node *JSDocReturnTag) VisitEachChild

// Go: ast/ast_generated.go:7487 (node *JSDocPublicTag) VisitEachChild

// Go: ast/ast_generated.go:7525 (node *JSDocPrivateTag) VisitEachChild

// Go: ast/ast_generated.go:7563 (node *JSDocProtectedTag) VisitEachChild

// Go: ast/ast_generated.go:7601 (node *JSDocReadonlyTag) VisitEachChild

// Go: ast/ast_generated.go:7639 (node *JSDocOverrideTag) VisitEachChild

// Go: ast/ast_generated.go:7677 (node *JSDocDeprecatedTag) VisitEachChild

// Go: ast/ast_generated.go:7717 (node *JSDocSeeTag) VisitEachChild

// Go: ast/ast_generated.go:7757 (node *JSDocImplementsTag) VisitEachChild

// Go: ast/ast_generated.go:7797 (node *JSDocAugmentsTag) VisitEachChild

// Go: ast/ast_generated.go:7837 (node *JSDocSatisfiesTag) VisitEachChild

// Go: ast/ast_generated.go:7877 (node *JSDocThrowsTag) VisitEachChild

// Go: ast/ast_generated.go:7917 (node *JSDocThisTag) VisitEachChild

// Go: ast/ast_generated.go:7965 (node *JSDocImportTag) VisitEachChild

// Go: ast/ast_generated.go:8010 (node *JSDocCallbackTag) VisitEachChild

// Go: ast/ast_generated.go:8054 (node *JSDocOverloadTag) VisitEachChild

// Go: ast/ast_generated.go:8099 (node *JSDocTypedefTag) VisitEachChild

// Go: ast/ast_generated.go:8143 (node *JSDocSignature) VisitEachChild

// Go: ast/ast_generated.go:8181 (node *JSDocNameReference) VisitEachChild

// Go: ast/ast_generated.go:8073 (node *ModuleDeclaration) VisitEachChild

// Go: ast/ast_generated.go:8293 (node *ImportEqualsDeclaration) VisitEachChild

// Go: ast/ast_generated.go:8348 (node *ExportDeclaration) VisitEachChild

// Go: ast/ast_generated.go:8396 (node *ImportTypeNode) VisitEachChild

// Go: ast/ast_generated.go:8441 (node *ImportClause) VisitEachChild

// Go: ast/ast_generated.go:8490 (node *ImportSpecifier) VisitEachChild

// Go: ast/ast_generated.go:8557 (node *JSDocLink) VisitEachChild

// Go: ast/ast_generated.go:8601 (node *JSDocLinkPlain) VisitEachChild

// Go: ast/ast_generated.go:8645 (node *JSDocLinkCode) VisitEachChild

// Go: ast/ast_generated.go:8701 (node *TypeParameterDeclaration) VisitEachChild

// Go: ast/ast_generated.go:8745 (node *SyntheticReferenceExpression) VisitEachChild

// Go: ast/ast_generated.go:8791 (node *JSDocTypeLiteral) VisitEachChild

// Go: ast/ast_generated.go:8838 (node *JSDocParameterOrPropertyTag) VisitEachChild
