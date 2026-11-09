//! Go `Clone` of AST nodes and lists (`ast/ast_generated.go` Clone methods,
//! `ast/ast.go` `cloneNode`, `NodeList.Clone`, `ModifierList.Clone`) and
//! `ast/deepclone.go`.
//!
//! Go dispatches `node.Clone(f)` on the node data type. Here
//! `NodeFactory::clone_node` matches on the astdata `NodeData` variant, which
//! has one variant per Go data type.

use crate::astdata::NodeData as D;
use crate::prelude::*;

// Go: ast/ast.go:114 cloneNode
// PORT: Go name `cloneNode` is `clone_node_from`, because
// `NodeFactory::clone_node` is Go `Node.Clone`.

impl NodeFactory {
    // Go: ast/ast.go:196 (n *Node) Clone

    // Go: ast/ast.go:147 (list *NodeList) Clone
    // PORT: a factory list fixes its `Loc` at creation, so it is passed to
    // `new_node_list_with_loc` instead of being set after `NewNodeList`.

    // Go: ast/ast.go:168 (list *ModifierList) Clone
    // PORT: Go copies `ModifierFlags`; `new_modifier_list_with_loc`
    // recomputes `ModifiersToFlags(nodes)`, which is the same value for a
    // list the factory or the parser made.

    // Go: ast/ast_generated.go:604 (node *Token) Clone

    // Go: ast/ast_generated.go:799 (node *Identifier) Clone

    // Go: ast/ast_generated.go:823 (node *PrivateIdentifier) Clone

    // Go: ast/ast_generated.go:865 (node *QualifiedName) Clone

    // Go: ast/ast_generated.go:909 (node *ComputedPropertyName) Clone

    // Go: ast/ast_generated.go:952 (node *Decorator) Clone

    // Go: ast/ast_generated.go:973 (node *EmptyStatement) Clone

    // Go: ast/ast_generated.go:1016 (node *IfStatement) Clone

    // Go: ast/ast_generated.go:1062 (node *DoStatement) Clone

    // Go: ast/ast_generated.go:1107 (node *WhileStatement) Clone

    // Go: ast/ast_generated.go:1160 (node *ForStatement) Clone

    // Go: ast/ast_generated.go:1216 (node *ForInOrOfStatement) Clone

    // Go: ast/ast_generated.go:1258 (node *BreakStatement) Clone

    // Go: ast/ast_generated.go:1296 (node *ContinueStatement) Clone

    // Go: ast/ast_generated.go:1335 (node *ReturnStatement) Clone

    // Go: ast/ast_generated.go:1376 (node *WithStatement) Clone

    // Go: ast/ast_generated.go:1422 (node *SwitchStatement) Clone

    // Go: ast/ast_generated.go:1467 (node *CaseBlock) Clone

    // Go: ast/ast_generated.go:1513 (node *CaseOrDefaultClause) Clone

    // Go: ast/ast_generated.go:1561 (node *ThrowStatement) Clone

    // Go: ast/ast_generated.go:1608 (node *TryStatement) Clone

    // Go: ast/ast_generated.go:1656 (node *CatchClause) Clone

    // Go: ast/ast_generated.go:1677 (node *DebuggerStatement) Clone

    // Go: ast/ast_generated.go:1717 (node *LabeledStatement) Clone

    // Go: ast/ast_generated.go:1760 (node *ExpressionStatement) Clone

    // Go: ast/ast_generated.go:1806 (node *Block) Clone

    // Go: ast/ast_generated.go:1851 (node *VariableStatement) Clone

    // Go: ast/ast_generated.go:1901 (node *VariableDeclaration) Clone

    // Go: ast/ast_generated.go:1946 (node *VariableDeclarationList) Clone

    // Go: ast/ast_generated.go:1985 (node *BindingPattern) Clone

    // Go: ast/ast_generated.go:2044 (node *ParameterDeclaration) Clone

    // Go: ast/ast_generated.go:2099 (node *BindingElement) Clone

    // Go: ast/ast_generated.go:2142 (node *MissingDeclaration) Clone

    // Go: ast/ast_generated.go:2200 (node *FunctionDeclaration) Clone

    // Go: ast/ast_generated.go:2251 (node *ClassDeclaration) Clone

    // Go: ast/ast_generated.go:2301 (node *ClassExpression) Clone

    // Go: ast/ast_generated.go:2346 (node *HeritageClause) Clone

    // Go: ast/ast_generated.go:2399 (node *InterfaceDeclaration) Clone

    // Go: ast/ast_generated.go:2470 (node *TypeAliasDeclaration) Clone

    // Go: ast/ast_generated.go:2526 (node *EnumMember) Clone

    // Go: ast/ast_generated.go:2575 (node *EnumDeclaration) Clone

    // Go: ast/ast_generated.go:2618 (node *ModuleBlock) Clone

    // Go: ast/ast_generated.go:2643 (node *NotEmittedStatement) Clone

    // Go: ast/ast_generated.go:2665 (node *NotEmittedTypeElement) Clone

    // Go: ast/ast_generated.go:2730 (node *ImportDeclaration) Clone

    // Go: ast/ast_generated.go:2786 (node *ExternalModuleReference) Clone

    // Go: ast/ast_generated.go:2830 (node *NamespaceImport) Clone

    // Go: ast/ast_generated.go:2877 (node *NamedImports) Clone

    // Go: ast/ast_generated.go:2927 (node *ExportAssignment) Clone

    // Go: ast/ast_generated.go:2969 (node *NamespaceExportDeclaration) Clone

    // Go: ast/ast_generated.go:3012 (node *NamespaceExport) Clone

    // Go: ast/ast_generated.go:3059 (node *NamedExports) Clone

    // Go: ast/ast_generated.go:3108 (node *ExportSpecifier) Clone

    // Go: ast/ast_generated.go:3155 (node *CallSignatureDeclaration) Clone

    // Go: ast/ast_generated.go:3198 (node *ConstructSignatureDeclaration) Clone

    // Go: ast/ast_generated.go:3251 (node *ConstructorDeclaration) Clone

    // Go: ast/ast_generated.go:3300 (node *GetAccessorDeclaration) Clone

    // Go: ast/ast_generated.go:3353 (node *SetAccessorDeclaration) Clone

    // Go: ast/ast_generated.go:3402 (node *IndexSignatureDeclaration) Clone

    // Go: ast/ast_generated.go:3453 (node *MethodSignatureDeclaration) Clone

    // Go: ast/ast_generated.go:3516 (node *MethodDeclaration) Clone

    // Go: ast/ast_generated.go:3570 (node *PropertySignatureDeclaration) Clone

    // Go: ast/ast_generated.go:3624 (node *PropertyDeclaration) Clone

    // Go: ast/ast_generated.go:3651 (node *SemicolonClassElement) Clone

    // Go: ast/ast_generated.go:3696 (node *ClassStaticBlockDeclaration) Clone

    // Go: ast/ast_generated.go:3717 (node *OmittedExpression) Clone

    // Go: ast/ast_generated.go:3739 (node *KeywordExpression) Clone

    // Go: ast/ast_generated.go:3772 (node *StringLiteral) Clone

    // Go: ast/ast_generated.go:3796 (node *NumericLiteral) Clone

    // Go: ast/ast_generated.go:3820 (node *BigIntLiteral) Clone

    // Go: ast/ast_generated.go:3844 (node *RegularExpressionLiteral) Clone

    // Go: ast/ast_generated.go:3870 (node *NoSubstitutionTemplateLiteral) Clone

    // Go: ast/ast_generated.go:3922 (node *BinaryExpression) Clone

    // Go: ast/ast_generated.go:3962 (node *PrefixUnaryExpression) Clone

    // Go: ast/ast_generated.go:4006 (node *PostfixUnaryExpression) Clone

    // Go: ast/ast_generated.go:4050 (node *YieldExpression) Clone

    // Go: ast/ast_generated.go:4105 (node *ArrowFunction) Clone

    // Go: ast/ast_generated.go:4163 (node *FunctionExpression) Clone

    // Go: ast/ast_generated.go:4207 (node *AsExpression) Clone

    // Go: ast/ast_generated.go:4247 (node *SatisfiesExpression) Clone

    // Go: ast/ast_generated.go:4298 (node *ConditionalExpression) Clone

    // Go: ast/ast_generated.go:4352 (node *PropertyAccessExpression) Clone

    // Go: ast/ast_generated.go:4402 (node *ElementAccessExpression) Clone

    // Go: ast/ast_generated.go:4459 (node *CallExpression) Clone

    // Go: ast/ast_generated.go:4502 (node *NewExpression) Clone

    // Go: ast/ast_generated.go:4544 (node *MetaProperty) Clone

    // Go: ast/ast_generated.go:4588 (node *NonNullExpression) Clone

    // Go: ast/ast_generated.go:4626 (node *SpreadElement) Clone

    // Go: ast/ast_generated.go:4667 (node *TemplateExpression) Clone

    // Go: ast/ast_generated.go:4712 (node *TemplateSpan) Clone

    // Go: ast/ast_generated.go:4767 (node *TaggedTemplateExpression) Clone

    // Go: ast/ast_generated.go:4805 (node *ParenthesizedExpression) Clone

    // Go: ast/ast_generated.go:4850 (node *ArrayLiteralExpression) Clone

    // Go: ast/ast_generated.go:4896 (node *ObjectLiteralExpression) Clone

    // Go: ast/ast_generated.go:4940 (node *SpreadAssignment) Clone

    // Go: ast/ast_generated.go:4990 (node *PropertyAssignment) Clone

    // Go: ast/ast_generated.go:5047 (node *ShorthandPropertyAssignment) Clone

    // Go: ast/ast_generated.go:5089 (node *DeleteExpression) Clone

    // Go: ast/ast_generated.go:5131 (node *TypeOfExpression) Clone

    // Go: ast/ast_generated.go:5173 (node *VoidExpression) Clone

    // Go: ast/ast_generated.go:5215 (node *AwaitExpression) Clone

    // Go: ast/ast_generated.go:5255 (node *TypeAssertion) Clone

    // Go: ast/ast_generated.go:5276 (node *KeywordTypeNode) Clone

    // Go: ast/ast_generated.go:5329 (node *UnionTypeNode) Clone

    // Go: ast/ast_generated.go:5367 (node *IntersectionTypeNode) Clone

    // Go: ast/ast_generated.go:5415 (node *ConditionalTypeNode) Clone

    // Go: ast/ast_generated.go:5455 (node *TypeOperatorNode) Clone

    // Go: ast/ast_generated.go:5493 (node *InferTypeNode) Clone

    // Go: ast/ast_generated.go:5531 (node *ArrayTypeNode) Clone

    // Go: ast/ast_generated.go:5571 (node *IndexedAccessTypeNode) Clone

    // Go: ast/ast_generated.go:5610 (node *TypeReferenceNode) Clone

    // Go: ast/ast_generated.go:5651 (node *ExpressionWithTypeArguments) Clone

    // Go: ast/ast_generated.go:5689 (node *LiteralTypeNode) Clone

    // Go: ast/ast_generated.go:5710 (node *ThisTypeNode) Clone

    // Go: ast/ast_generated.go:5752 (node *TypePredicateNode) Clone

    // Go: ast/ast_generated.go:5793 (node *ImportAttribute) Clone

    // Go: ast/ast_generated.go:5845 (node *ImportAttributes) Clone

    // Go: ast/ast_generated.go:5888 (node *TypeQueryNode) Clone

    // Go: ast/ast_generated.go:5943 (node *MappedTypeNode) Clone

    // Go: ast/ast_generated.go:5982 (node *TypeLiteralNode) Clone

    // Go: ast/ast_generated.go:6020 (node *TupleTypeNode) Clone

    // Go: ast/ast_generated.go:6068 (node *NamedTupleMember) Clone

    // Go: ast/ast_generated.go:6110 (node *OptionalTypeNode) Clone

    // Go: ast/ast_generated.go:6148 (node *RestTypeNode) Clone

    // Go: ast/ast_generated.go:6186 (node *ParenthesizedTypeNode) Clone

    // Go: ast/ast_generated.go:6226 (node *FunctionTypeNode) Clone

    // Go: ast/ast_generated.go:6270 (node *ConstructorTypeNode) Clone

    // Go: ast/ast_generated.go:6296 (node *TemplateHead) Clone

    // Go: ast/ast_generated.go:6322 (node *TemplateMiddle) Clone

    // Go: ast/ast_generated.go:6348 (node *TemplateTail) Clone

    // Go: ast/ast_generated.go:6388 (node *TemplateLiteralTypeNode) Clone

    // Go: ast/ast_generated.go:6428 (node *TemplateLiteralTypeSpan) Clone

    // Go: ast/ast_generated.go:6470 (node *SyntheticExpression) Clone

    // Go: ast/ast_generated.go:6508 (node *PartiallyEmittedExpression) Clone

    // Go: ast/ast_generated.go:6555 (node *JsxElement) Clone

    // Go: ast/ast_generated.go:6595 (node *JsxAttributes) Clone

    // Go: ast/ast_generated.go:6636 (node *JsxNamespacedName) Clone

    // Go: ast/ast_generated.go:6683 (node *JsxOpeningElement) Clone

    // Go: ast/ast_generated.go:6726 (node *JsxSelfClosingElement) Clone

    // Go: ast/ast_generated.go:6769 (node *JsxFragment) Clone

    // Go: ast/ast_generated.go:6790 (node *JsxOpeningFragment) Clone

    // Go: ast/ast_generated.go:6811 (node *JsxClosingFragment) Clone

    // Go: ast/ast_generated.go:6853 (node *JsxAttribute) Clone

    // Go: ast/ast_generated.go:6896 (node *JsxSpreadAttribute) Clone

    // Go: ast/ast_generated.go:6934 (node *JsxClosingElement) Clone

    // Go: ast/ast_generated.go:6974 (node *JsxExpression) Clone

    // Go: ast/ast_generated.go:7000 (node *JsxText) Clone

    // Go: ast/ast_generated.go:7039 (node *SyntaxList) Clone

    // Go: ast/ast_generated.go:7079 (node *JSDoc) Clone

    // Go: ast/ast_generated.go:7117 (node *JSDocTypeExpression) Clone

    // Go: ast/ast_generated.go:7155 (node *JSDocNonNullableType) Clone

    // Go: ast/ast_generated.go:7193 (node *JSDocNullableType) Clone

    // Go: ast/ast_generated.go:7214 (node *JSDocAllType) Clone

    // Go: ast/ast_generated.go:7252 (node *JSDocVariadicType) Clone

    // Go: ast/ast_generated.go:7290 (node *JSDocOptionalType) Clone

    // Go: ast/ast_generated.go:7330 (node *JSDocTypeTag) Clone

    // Go: ast/ast_generated.go:7368 (node *JSDocUnknownTag) Clone

    // Go: ast/ast_generated.go:7413 (node *JSDocTemplateTag) Clone

    // Go: ast/ast_generated.go:7453 (node *JSDocReturnTag) Clone

    // Go: ast/ast_generated.go:7491 (node *JSDocPublicTag) Clone

    // Go: ast/ast_generated.go:7529 (node *JSDocPrivateTag) Clone

    // Go: ast/ast_generated.go:7567 (node *JSDocProtectedTag) Clone

    // Go: ast/ast_generated.go:7605 (node *JSDocReadonlyTag) Clone

    // Go: ast/ast_generated.go:7643 (node *JSDocOverrideTag) Clone

    // Go: ast/ast_generated.go:7681 (node *JSDocDeprecatedTag) Clone

    // Go: ast/ast_generated.go:7721 (node *JSDocSeeTag) Clone

    // Go: ast/ast_generated.go:7761 (node *JSDocImplementsTag) Clone

    // Go: ast/ast_generated.go:7801 (node *JSDocAugmentsTag) Clone

    // Go: ast/ast_generated.go:7841 (node *JSDocSatisfiesTag) Clone

    // Go: ast/ast_generated.go:7881 (node *JSDocThrowsTag) Clone

    // Go: ast/ast_generated.go:7921 (node *JSDocThisTag) Clone

    // Go: ast/ast_generated.go:7969 (node *JSDocImportTag) Clone

    // Go: ast/ast_generated.go:8014 (node *JSDocCallbackTag) Clone

    // Go: ast/ast_generated.go:8058 (node *JSDocOverloadTag) Clone

    // Go: ast/ast_generated.go:8103 (node *JSDocTypedefTag) Clone

    // Go: ast/ast_generated.go:8147 (node *JSDocSignature) Clone

    // Go: ast/ast_generated.go:8185 (node *JSDocNameReference) Clone

    // Go: ast/ast_generated.go:8077 (node *ModuleDeclaration) Clone

    // Go: ast/ast_generated.go:8297 (node *ImportEqualsDeclaration) Clone

    // Go: ast/ast_generated.go:8352 (node *ExportDeclaration) Clone

    // Go: ast/ast_generated.go:8400 (node *ImportTypeNode) Clone

    // Go: ast/ast_generated.go:8445 (node *ImportClause) Clone

    // Go: ast/ast_generated.go:8494 (node *ImportSpecifier) Clone

    // Go: ast/ast_generated.go:8521 (node *JSDocText) Clone

    // Go: ast/ast_generated.go:8561 (node *JSDocLink) Clone

    // Go: ast/ast_generated.go:8605 (node *JSDocLinkPlain) Clone

    // Go: ast/ast_generated.go:8649 (node *JSDocLinkCode) Clone

    // Go: ast/ast_generated.go:8705 (node *TypeParameterDeclaration) Clone

    // Go: ast/ast_generated.go:8749 (node *SyntheticReferenceExpression) Clone

    // Go: ast/ast_generated.go:8796 (node *JSDocTypeLiteral) Clone

    // Go: ast/ast_generated.go:8842 (node *JSDocParameterOrPropertyTag) Clone

    // Go: ast/ast.go:2682 (node *SourceFile) Clone
}

// Go: ast/deepclone.go:6 getDeepCloneVisitor
// Ideally, this would get cached on the node factory so there's only ever one set of closures made per factory
// PORT: Go sets `newList.Loc` in place. A synthetic list fixes its `Loc` when
// it is made, so the hooks make a new list with the same nodes and the new
// `Loc`. The returned list is the same as in Go.

impl NodeFactory {
    // Go: ast/deepclone.go:71 DeepCloneNode

    // Go: ast/deepclone.go:75 DeepCloneReparse

    // Go: ast/deepclone.go:84 DeepCloneReparseModifiers
}

#[cfg(test)]
mod tests {
    use crate::ast::synthetic::synthetic_slot_count;
    use crate::frontend::parser::{
        SourceFileParseOptions, parse_source_file, parse_source_file_detached,
    };
    use crate::prelude::*;

    /// JSDoc that the reparser deep clones with lists of nodes: `@import`,
    /// `@overload`, a function type in `@type`, an object type in `@typedef`.
    const TEXT: &str = "/** @import { a } from \"./b.js\" */\n\
        /** @overload @param {number} x @returns {void} */\n\
        /** @param {any} x */\nexport function o(x) {}\n\
        /** @type {(x: number) => void} */\nexport const fn = (x) => {};\n\
        /** @typedef {{x: number}} T */\n";

    fn opts() -> SourceFileParseOptions {
        SourceFileParseOptions {
            file_name: "/a.js".to_string(),
            ..Default::default()
        }
    }

    /// A parse worker's parse of `TEXT` makes no synthetic slot and no id,
    /// so the loader can adopt it (`files_parser.rs` `prefetch_parse`). Go
    /// makes those lists with the parser's factory (ast/visitor.go
    /// `VisitNodes`, `VisitModifiers`).
    #[test]
    fn reparsed_jsdoc_lists_stay_in_the_detached_store() {
        std::thread::spawn(|| {
            let before = (synthetic_slot_count(), crate::ast::utilities_p1::next_ids());
            let parse = parse_source_file_detached(0, &opts(), TEXT, ScriptKind::JS);
            assert!(parse.store.is_self_contained());
            assert_eq!(
                (synthetic_slot_count(), crate::ast::utilities_p1::next_ids()),
                before
            );
        })
        .join()
        .unwrap();
    }

    /// A language server parse of `TEXT` (a freeable parse, whose store owns
    /// its nodes) puts those lists in its store too, not in the thread's
    /// synthetic arena, so they are freed with the file version.
    #[test]
    fn reparsed_jsdoc_lists_stay_in_the_owned_store() {
        std::thread::spawn(|| {
            let before = synthetic_slot_count();
            let file = {
                let _scope = crate::ast::enter_owned_parse();
                parse_source_file(&opts(), TEXT, ScriptKind::JS)
            };
            assert!(crate::ast::try_store_ast_node(file.root).is_none(), "owned");
            assert_eq!(synthetic_slot_count(), before);
        })
        .join()
        .unwrap();
    }
}
