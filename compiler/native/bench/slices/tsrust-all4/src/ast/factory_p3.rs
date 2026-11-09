//! More Go `ast.NodeFactory` constructors (`ast/ast_generated.go`) that
//! `ast/factory.rs` does not have. `ast/update.rs` and `ast/clone.rs` call
//! them.
//!
//! The argument rules are the same as in `ast/factory.rs`: Go `*Node` is
//! `Node`, Go `*NodeList` is `NodeList`, Go `*ModifierList` is
//! `ModifierList`, and Go `nil` is the `NIL` value of each. Nodes go to the
//! factory target (synthetic arena or the store of the parsed file).

use crate::astdata::NodeData as D;
use crate::prelude::*;

/// Go `TokenFlagsNone` in astdata form.
const NO_TOKEN_FLAGS: crate::astdata::TokenFlags = crate::astdata::TokenFlags(0);

impl NodeFactory {
    // Go: ast/ast_generated.go:875 NewDoStatement

    // Go: ast/ast_generated.go:920 NewWhileStatement

    // Go: ast/ast_generated.go:968 NewForStatement

    // Go: ast/ast_generated.go:1024 NewForInOrOfStatement

    // Go: ast/ast_generated.go:1072 NewBreakStatement

    // Go: ast/ast_generated.go:1110 NewContinueStatement

    // Go: ast/ast_generated.go:1189 NewWithStatement

    // Go: ast/ast_generated.go:1235 NewSwitchStatement

    // Go: ast/ast_generated.go:1281 NewCaseBlock

    // Go: ast/ast_generated.go:1326 NewCaseOrDefaultClause

    // Go: ast/ast_generated.go:1375 NewThrowStatement

    // Go: ast/ast_generated.go:1420 NewTryStatement

    // Go: ast/ast_generated.go:1469 NewCatchClause

    // Go: ast/ast_generated.go:1507 NewDebuggerStatement

    // Go: ast/ast_generated.go:1530 NewLabeledStatement

    // Go: ast/ast_generated.go:1799 NewBindingPattern

    // Go: ast/ast_generated.go:1907 NewBindingElement

    // Go: ast/ast_generated.go:1956 NewMissingDeclaration

    // Go: ast/ast_generated.go:2272 NewJSTypeAliasDeclaration

    // Go: ast/ast_generated.go:2534 NewJSImportDeclaration

    // Go: ast/ast_generated.go:2647 NewNamespaceImport

    // Go: ast/ast_generated.go:2785 NewNamespaceExportDeclaration

    // Go: ast/ast_generated.go:2829 NewNamespaceExport

    // Go: ast/ast_generated.go:3488 NewSemicolonClassElement

    // Go: ast/ast_generated.go:3516 NewClassStaticBlockDeclaration

    // Go: ast/ast_generated.go:3554 NewOmittedExpression

    // Go: ast/ast_generated.go:3821 NewPostfixUnaryExpression

    // Go: ast/ast_generated.go:4359 NewMetaProperty

    // Go: ast/ast_generated.go:4482 NewTemplateExpression

    // Go: ast/ast_generated.go:4527 NewTemplateSpan

    // Go: ast/ast_generated.go:4756 NewSpreadAssignment

    // Go: ast/ast_generated.go:4855 NewShorthandPropertyAssignment

    // Go: ast/ast_generated.go:6311 NewPartiallyEmittedExpression

    // Go: ast/ast_generated.go:6356 NewJsxElement

    // Go: ast/ast_generated.go:6398 NewJsxAttributes

    // Go: ast/ast_generated.go:6438 NewJsxNamespacedName

    // Go: ast/ast_generated.go:6484 NewJsxOpeningElement

    // Go: ast/ast_generated.go:6527 NewJsxSelfClosingElement

    // Go: ast/ast_generated.go:6570 NewJsxFragment

    // Go: ast/ast_generated.go:6609 NewJsxOpeningFragment

    // Go: ast/ast_generated.go:6630 NewJsxClosingFragment

    // Go: ast/ast_generated.go:6655 NewJsxAttribute

    // Go: ast/ast_generated.go:6700 NewJsxSpreadAttribute

    // Go: ast/ast_generated.go:6738 NewJsxClosingElement

    // Go: ast/ast_generated.go:6777 NewJsxExpression

    // Go: ast/ast_generated.go:6817 NewJsxText

    // Go: ast/ast_generated.go:6882 NewJSDoc

    // Go: ast/ast_generated.go:6921 NewJSDocTypeExpression

    // Go: ast/ast_generated.go:6959 NewJSDocNonNullableType

    // Go: ast/ast_generated.go:6997 NewJSDocNullableType

    // Go: ast/ast_generated.go:7034 NewJSDocAllType

    // Go: ast/ast_generated.go:7056 NewJSDocVariadicType

    // Go: ast/ast_generated.go:7094 NewJSDocOptionalType

    // Go: ast/ast_generated.go:7132 NewJSDocTypeTag

    // Go: ast/ast_generated.go:7171 NewJSDocUnknownTag

    // Go: ast/ast_generated.go:7211 NewJSDocTemplateTag

    // Go: ast/ast_generated.go:7255 NewJSDocReturnTag

    // Go: ast/ast_generated.go:7294 NewJSDocPublicTag

    // Go: ast/ast_generated.go:7332 NewJSDocPrivateTag

    // Go: ast/ast_generated.go:7370 NewJSDocProtectedTag

    // Go: ast/ast_generated.go:7408 NewJSDocReadonlyTag

    // Go: ast/ast_generated.go:7446 NewJSDocOverrideTag

    // Go: ast/ast_generated.go:7484 NewJSDocDeprecatedTag

    // Go: ast/ast_generated.go:7523 NewJSDocSeeTag

    // Go: ast/ast_generated.go:7563 NewJSDocImplementsTag

    // Go: ast/ast_generated.go:7603 NewJSDocAugmentsTag

    // Go: ast/ast_generated.go:7643 NewJSDocSatisfiesTag

    // Go: ast/ast_generated.go:7683 NewJSDocThrowsTag

    // Go: ast/ast_generated.go:7723 NewJSDocThisTag

    // Go: ast/ast_generated.go:7765 NewJSDocImportTag

    // Go: ast/ast_generated.go:7812 NewJSDocCallbackTag

    // Go: ast/ast_generated.go:7860 NewJSDocOverloadTag

    // Go: ast/ast_generated.go:7901 NewJSDocTypedefTag

    // Go: ast/ast_generated.go:7950 NewJSDocSignature

    // Go: ast/ast_generated.go:7990 NewJSDocNameReference

    // Go: ast/ast_generated.go:8345 NewJSDocText

    // Go: ast/ast_generated.go:8369 NewJSDocLink

    // Go: ast/ast_generated.go:8413 NewJSDocLinkPlain

    // Go: ast/ast_generated.go:8457 NewJSDocLinkCode

    // Go: ast/ast_generated.go:8558 NewSyntheticReferenceExpression

    // Go: ast/ast_generated.go:8604 NewJSDocTypeLiteral
    // PORT: a Go nil or empty slice is `None`; Go code only ranges over it.

    // Go: ast/ast_generated.go:8647 NewJSDocParameterOrPropertyTag
}
