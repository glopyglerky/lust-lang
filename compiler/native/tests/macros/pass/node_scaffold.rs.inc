// Shared scaffolding for the node macro fixtures. This file is
// include!d by ts_node_accessors.rs, ts_node_walk.rs and
// ts_with_function_like.rs. It is not a standalone test.

// ---------- minimal Node universe (scaffolding, not macro text) ----------
// Included by the node macro fixtures with include!. The shape follows
// ts-rust: NodeData variants hold boxed per-variant data structs, Node
// is a copyable handle into a leaked static store, and the module
// paths $crate::core, $crate::astdata and $crate::ast::synthetic
// resolve as the copied macros expect.

pub mod astdata {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct NodeId(pub u32);
    impl NodeId { pub fn index(self) -> usize { self.0 as usize } }
    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct NodeList { pub items: Vec<NodeId> }
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct ModFlags(pub u32);
    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct ModifierList { pub list: NodeList, pub flags: ModFlags }
    pub use crate::NodeData;
}
pub mod core { pub use crate::Node; }
pub mod ast {
    pub use crate::{ModifierList, NodeList};
    pub mod synthetic {
        use crate::{Node, NodeData};
        #[derive(Clone, Copy)]
        pub enum AnyList<'a> {
            Nodes(&'a crate::astdata::NodeList),
            Modifiers(&'a crate::astdata::ModifierList),
        }
        impl<'a> AnyList<'a> {
            pub fn nodes(self) -> &'a crate::astdata::NodeList {
                match self { Self::Nodes(l) => l, Self::Modifiers(m) => &m.list }
            }
            pub fn modifiers(self) -> &'a crate::astdata::ModifierList {
                match self { Self::Modifiers(m) => m, Self::Nodes(_) => panic!("node list as modifiers") }
            }
        }
        pub struct SelectorSite;
        impl SelectorSite { pub const fn new() -> Self { SelectorSite } }
        pub fn static_tier_ast_node(n: Node) -> Option<&'static NodeData> { crate::static_ast_node(n) }
        pub fn read_ast_node_miss<R>(n: Node, read: impl FnOnce(&NodeData) -> R) -> R {
            read(crate::static_ast_node(n).expect("registered node"))
        }
        pub fn scoped_node_list_of(n: Node, _site: &SelectorSite, sel: impl FnOnce(&NodeData) -> Option<Option<AnyList<'_>>>) -> Option<crate::NodeList> {
            let d = crate::static_ast_node(n)?;
            sel(d).map(|l| crate::NodeList::from_ts(n.file_index(), l.map(AnyList::nodes)))
        }
        pub fn scoped_modifiers_of(n: Node, _site: &SelectorSite, sel: impl FnOnce(&NodeData) -> Option<Option<AnyList<'_>>>) -> Option<crate::ModifierList> {
            let d = crate::static_ast_node(n)?;
            sel(d).map(|l| crate::ModifierList::from_ts(n.file_index(), l.map(AnyList::modifiers)))
        }
    }
}
pub type LoadedData = Option<&'static NodeData>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyntaxKind { ArrayBindingPattern, ArrayType, ArrowFunction, AsExpression, AwaitExpression, BinaryExpression, BindingElement, Block, CallExpression, CallSignature, CallSignatureDeclaration, CaseBlock, CaseClause, CaseOrDefaultClause, CatchClause, ClassDeclaration, ClassExpression, ClassStaticBlockDeclaration, ComputedPropertyName, ConditionalExpression, ConditionalType, ConditionalTypeNode, ConstructSignature, ConstructSignatureDeclaration, Constructor, ConstructorDeclaration, ConstructorType, ConstructorTypeNode, Decorator, DefaultClause, DeleteExpression, DoStatement, ElementAccessExpression, EnumDeclaration, EnumMember, ExportAssignment, ExportDeclaration, ExportSpecifier, ExpressionStatement, ExpressionWithTypeArguments, ExternalModuleReference, ForInOrOfStatement, ForInStatement, ForOfStatement, ForStatement, FunctionDeclaration, FunctionExpression, FunctionType, FunctionTypeNode, GetAccessor, GetAccessorDeclaration, Identifier, IfStatement, ImportAttribute, ImportClause, ImportDeclaration, ImportEqualsDeclaration, ImportSpecifier, ImportType, ImportTypeNode, IndexSignature, IndexSignatureDeclaration, IndexedAccessType, InferType, InterfaceDeclaration, IntersectionType, JsDocCallbackTag, JsDocLink, JsDocLinkCode, JsDocLinkPlain, JsDocNameReference, JsDocNonNullableType, JsDocNullableType, JsDocOptionalType, JsDocParameterOrPropertyTag, JsDocParameterTag, JsDocPropertyTag, JsDocSignature, JsDocTypeExpression, JsDocTypedefTag, JsDocVariadicType, JsImportDeclaration, JsTypeAliasDeclaration, JsxAttribute, JsxExpression, JsxNamespacedName, JsxOpeningElement, JsxSelfClosingElement, JsxSpreadAttribute, JsxText, JsxTextAllWhiteSpaces, LiteralType, MappedType, MappedTypeNode, MetaProperty, MethodDeclaration, MethodSignature, MethodSignatureDeclaration, MissingDeclaration, ModuleDeclaration, NamedTupleMember, NamespaceExport, NamespaceExportDeclaration, NamespaceImport, NewExpression, NonNullExpression, ObjectBindingPattern, OptionalType, OptionalTypeNode, Parameter, ParameterDeclaration, ParenthesizedExpression, ParenthesizedType, ParenthesizedTypeNode, PartiallyEmittedExpression, PropertyAccessExpression, PropertyAssignment, PropertyDeclaration, PropertySignature, PropertySignatureDeclaration, QualifiedName, RestType, RestTypeNode, ReturnStatement, SatisfiesExpression, SetAccessor, SetAccessorDeclaration, ShorthandPropertyAssignment, SourceFile, SpreadAssignment, SpreadElement, SwitchStatement, SyntheticReferenceExpression, TaggedTemplateExpression, TemplateLiteralType, TemplateLiteralTypeSpan, TemplateSpan, ThisType, ThrowStatement, Token, TupleType, TypeAliasDeclaration, TypeAssertion, TypeAssertionExpression, TypeLiteral, TypeOfExpression, TypeOperator, TypeOperatorNode, TypeParameter, TypeParameterDeclaration, TypePredicate, TypePredicateNode, TypeQuery, TypeQueryNode, TypeReference, TypeReferenceNode, UnionType, Unknown, VariableDeclaration, VariableStatement, VoidExpression, WhileStatement, WithStatement, YieldExpression }
impl SyntaxKind {
    pub fn is_token(self) -> bool { matches!(self, SyntaxKind::Identifier) }
    pub fn is_keyword_expression(self) -> bool { matches!(self, SyntaxKind::Identifier) }
    pub fn is_keyword_type(self) -> bool { matches!(self, SyntaxKind::TypeReference) }
}

#[derive(Debug, Default)]
pub struct ArrowFunctionData {
    pub value: i32,
    pub type_: Option<crate::astdata::NodeId>,
    pub modifiers: Option<crate::astdata::ModifierList>,
    pub parameters: crate::astdata::NodeList,
}
#[derive(Debug, Default)]
pub struct AsExpressionData {
    pub value: i32,
    pub expression: crate::astdata::NodeId,
    pub type_: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct AwaitExpressionData {
    pub value: i32,
    pub expression: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct BinaryExpressionData {
    pub value: i32,
    pub type_: Option<crate::astdata::NodeId>,
    pub modifiers: Option<crate::astdata::ModifierList>,
    pub right: crate::astdata::NodeId,
    pub left: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct BindingElementData {
    pub value: i32,
    pub name: Option<crate::astdata::NodeId>,
    pub initializer: Option<crate::astdata::NodeId>,
}
#[derive(Debug, Default)]
pub struct BlockData {
    pub value: i32,
}
#[derive(Debug, Default)]
pub struct CallExpressionData {
    pub value: i32,
    pub expression: crate::astdata::NodeId,
    pub type_arguments: Option<crate::astdata::NodeList>,
    pub question_dot_token: Option<crate::astdata::NodeId>,
}
#[derive(Debug, Default)]
pub struct CallSignatureDeclarationData {
    pub value: i32,
    pub type_: Option<crate::astdata::NodeId>,
    pub parameters: crate::astdata::NodeList,
}
#[derive(Debug, Default)]
pub struct CaseBlockData {
    pub value: i32,
}
#[derive(Debug, Default)]
pub struct CaseOrDefaultClauseData {
    pub value: i32,
}
#[derive(Debug, Default)]
pub struct CatchClauseData {
    pub value: i32,
}
#[derive(Debug, Default)]
pub struct ClassDeclarationData {
    pub value: i32,
    pub name: Option<crate::astdata::NodeId>,
    pub modifiers: Option<crate::astdata::ModifierList>,
}
#[derive(Debug, Default)]
pub struct ClassExpressionData {
    pub value: i32,
    pub name: Option<crate::astdata::NodeId>,
    pub modifiers: Option<crate::astdata::ModifierList>,
}
#[derive(Debug, Default)]
pub struct ClassStaticBlockDeclarationData {
    pub value: i32,
    pub modifiers: Option<crate::astdata::ModifierList>,
}
#[derive(Debug, Default)]
pub struct ComputedPropertyNameData {
    pub value: i32,
    pub expression: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct ConditionalExpressionData {
    pub value: i32,
    pub question_token: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct ConditionalTypeNodeData {
    pub value: i32,
}
#[derive(Debug, Default)]
pub struct ConstructSignatureDeclarationData {
    pub value: i32,
    pub type_: Option<crate::astdata::NodeId>,
    pub parameters: crate::astdata::NodeList,
}
#[derive(Debug, Default)]
pub struct ConstructorDeclarationData {
    pub value: i32,
    pub type_: Option<crate::astdata::NodeId>,
    pub modifiers: Option<crate::astdata::ModifierList>,
    pub parameters: crate::astdata::NodeList,
}
#[derive(Debug, Default)]
pub struct ConstructorTypeNodeData {
    pub value: i32,
    pub type_: Option<crate::astdata::NodeId>,
    pub modifiers: Option<crate::astdata::ModifierList>,
    pub parameters: crate::astdata::NodeList,
}
#[derive(Debug, Default)]
pub struct DecoratorData {
    pub value: i32,
    pub expression: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct DeleteExpressionData {
    pub value: i32,
    pub expression: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct DoStatementData {
    pub value: i32,
    pub expression: crate::astdata::NodeId,
    pub statement: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct ElementAccessExpressionData {
    pub value: i32,
    pub expression: crate::astdata::NodeId,
    pub question_dot_token: Option<crate::astdata::NodeId>,
}
#[derive(Debug, Default)]
pub struct EnumDeclarationData {
    pub value: i32,
    pub name: crate::astdata::NodeId,
    pub modifiers: Option<crate::astdata::ModifierList>,
}
#[derive(Debug, Default)]
pub struct EnumMemberData {
    pub value: i32,
    pub name: crate::astdata::NodeId,
    pub postfix_token: Option<crate::astdata::NodeId>,
    pub initializer: Option<crate::astdata::NodeId>,
    pub modifiers: Option<crate::astdata::ModifierList>,
}
#[derive(Debug, Default)]
pub struct ExportAssignmentData {
    pub value: i32,
    pub expression: crate::astdata::NodeId,
    pub type_: Option<crate::astdata::NodeId>,
    pub modifiers: Option<crate::astdata::ModifierList>,
}
#[derive(Debug, Default)]
pub struct ExportDeclarationData {
    pub value: i32,
    pub modifiers: Option<crate::astdata::ModifierList>,
}
#[derive(Debug, Default)]
pub struct ExportSpecifierData {
    pub value: i32,
    pub name: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct ExpressionStatementData {
    pub value: i32,
    pub expression: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct ExpressionWithTypeArgumentsData {
    pub value: i32,
    pub expression: crate::astdata::NodeId,
    pub type_arguments: Option<crate::astdata::NodeList>,
}
#[derive(Debug, Default)]
pub struct ExternalModuleReferenceData {
    pub value: i32,
    pub expression: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct ForInOrOfStatementData {
    pub value: i32,
    pub expression: crate::astdata::NodeId,
    pub initializer: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct ForStatementData {
    pub value: i32,
    pub initializer: Option<crate::astdata::NodeId>,
}
#[derive(Debug, Default)]
pub struct FunctionDeclarationData {
    pub value: i32,
    pub name: Option<crate::astdata::NodeId>,
    pub type_: Option<crate::astdata::NodeId>,
    pub modifiers: Option<crate::astdata::ModifierList>,
    pub parameters: crate::astdata::NodeList,
}
#[derive(Debug, Default)]
pub struct FunctionExpressionData {
    pub value: i32,
    pub name: Option<crate::astdata::NodeId>,
    pub type_: Option<crate::astdata::NodeId>,
    pub modifiers: Option<crate::astdata::ModifierList>,
    pub parameters: crate::astdata::NodeList,
}
#[derive(Debug, Default)]
pub struct FunctionTypeNodeData {
    pub value: i32,
    pub type_: Option<crate::astdata::NodeId>,
    pub modifiers: Option<crate::astdata::ModifierList>,
    pub parameters: crate::astdata::NodeList,
}
#[derive(Debug, Default)]
pub struct GetAccessorDeclarationData {
    pub value: i32,
    pub name: crate::astdata::NodeId,
    pub postfix_token: Option<crate::astdata::NodeId>,
    pub type_: Option<crate::astdata::NodeId>,
    pub modifiers: Option<crate::astdata::ModifierList>,
    pub parameters: crate::astdata::NodeList,
}
#[derive(Debug, Default)]
pub struct IdentifierData {
    pub value: i32,
}
#[derive(Debug, Default)]
pub struct IfStatementData {
    pub value: i32,
    pub expression: crate::astdata::NodeId,
    pub then_statement: crate::astdata::NodeId,
    pub else_statement: Option<crate::astdata::NodeId>,
}
#[derive(Debug, Default)]
pub struct ImportAttributeData {
    pub value: i32,
    pub name: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct ImportClauseData {
    pub value: i32,
    pub name: Option<crate::astdata::NodeId>,
}
#[derive(Debug, Default)]
pub struct ImportDeclarationData {
    pub value: i32,
    pub modifiers: Option<crate::astdata::ModifierList>,
}
#[derive(Debug, Default)]
pub struct ImportEqualsDeclarationData {
    pub value: i32,
    pub name: crate::astdata::NodeId,
    pub modifiers: Option<crate::astdata::ModifierList>,
}
#[derive(Debug, Default)]
pub struct ImportSpecifierData {
    pub value: i32,
    pub name: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct ImportTypeNodeData {
    pub value: i32,
    pub type_arguments: Option<crate::astdata::NodeList>,
}
#[derive(Debug, Default)]
pub struct IndexSignatureDeclarationData {
    pub value: i32,
    pub type_: crate::astdata::NodeId,
    pub modifiers: Option<crate::astdata::ModifierList>,
    pub parameters: crate::astdata::NodeList,
}
#[derive(Debug, Default)]
pub struct InterfaceDeclarationData {
    pub value: i32,
    pub name: crate::astdata::NodeId,
    pub modifiers: Option<crate::astdata::ModifierList>,
}
#[derive(Debug, Default)]
pub struct JsDocCallbackTagData {
    pub value: i32,
    pub name: Option<crate::astdata::NodeId>,
}
#[derive(Debug, Default)]
pub struct JsDocLinkData {
    pub value: i32,
    pub name: Option<crate::astdata::NodeId>,
}
#[derive(Debug, Default)]
pub struct JsDocLinkCodeData {
    pub value: i32,
    pub name: Option<crate::astdata::NodeId>,
}
#[derive(Debug, Default)]
pub struct JsDocLinkPlainData {
    pub value: i32,
    pub name: Option<crate::astdata::NodeId>,
}
#[derive(Debug, Default)]
pub struct JsDocNameReferenceData {
    pub value: i32,
    pub name: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct JsDocNonNullableTypeData {
    pub value: i32,
    pub type_: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct JsDocNullableTypeData {
    pub value: i32,
    pub type_: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct JsDocOptionalTypeData {
    pub value: i32,
    pub type_: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct JsDocParameterOrPropertyTagData {
    pub value: i32,
    pub name: crate::astdata::NodeId,
    pub type_expression: Option<crate::astdata::NodeId>,
}
#[derive(Debug, Default)]
pub struct JsDocSignatureData {
    pub value: i32,
    pub type_: Option<crate::astdata::NodeId>,
    pub parameters: crate::astdata::NodeList,
}
#[derive(Debug, Default)]
pub struct JsDocTypeExpressionData {
    pub value: i32,
    pub type_: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct JsDocTypedefTagData {
    pub value: i32,
    pub name: Option<crate::astdata::NodeId>,
}
#[derive(Debug, Default)]
pub struct JsDocVariadicTypeData {
    pub value: i32,
    pub type_: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct JsxAttributeData {
    pub value: i32,
    pub name: crate::astdata::NodeId,
    pub initializer: Option<crate::astdata::NodeId>,
}
#[derive(Debug, Default)]
pub struct JsxExpressionData {
    pub value: i32,
    pub expression: Option<crate::astdata::NodeId>,
}
#[derive(Debug, Default)]
pub struct JsxNamespacedNameData {
    pub value: i32,
    pub name: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct JsxOpeningElementData {
    pub value: i32,
    pub type_arguments: Option<crate::astdata::NodeList>,
}
#[derive(Debug, Default)]
pub struct JsxSelfClosingElementData {
    pub value: i32,
    pub type_arguments: Option<crate::astdata::NodeList>,
}
#[derive(Debug, Default)]
pub struct JsxSpreadAttributeData {
    pub value: i32,
    pub expression: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct MappedTypeNodeData {
    pub value: i32,
    pub question_token: Option<crate::astdata::NodeId>,
    pub type_: Option<crate::astdata::NodeId>,
}
#[derive(Debug, Default)]
pub struct MetaPropertyData {
    pub value: i32,
    pub name: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct MethodDeclarationData {
    pub value: i32,
    pub name: crate::astdata::NodeId,
    pub postfix_token: Option<crate::astdata::NodeId>,
    pub type_: Option<crate::astdata::NodeId>,
    pub modifiers: Option<crate::astdata::ModifierList>,
    pub parameters: crate::astdata::NodeList,
}
#[derive(Debug, Default)]
pub struct MethodSignatureDeclarationData {
    pub value: i32,
    pub name: crate::astdata::NodeId,
    pub postfix_token: Option<crate::astdata::NodeId>,
    pub type_: Option<crate::astdata::NodeId>,
    pub modifiers: Option<crate::astdata::ModifierList>,
    pub parameters: crate::astdata::NodeList,
}
#[derive(Debug, Default)]
pub struct MissingDeclarationData {
    pub value: i32,
    pub modifiers: Option<crate::astdata::ModifierList>,
}
#[derive(Debug, Default)]
pub struct ModuleDeclarationData {
    pub value: i32,
    pub name: crate::astdata::NodeId,
    pub modifiers: Option<crate::astdata::ModifierList>,
}
#[derive(Debug, Default)]
pub struct NamedTupleMemberData {
    pub value: i32,
    pub name: crate::astdata::NodeId,
    pub question_token: Option<crate::astdata::NodeId>,
    pub type_: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct NamespaceExportData {
    pub value: i32,
    pub name: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct NamespaceExportDeclarationData {
    pub value: i32,
    pub name: crate::astdata::NodeId,
    pub modifiers: Option<crate::astdata::ModifierList>,
}
#[derive(Debug, Default)]
pub struct NamespaceImportData {
    pub value: i32,
    pub name: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct NewExpressionData {
    pub value: i32,
    pub expression: crate::astdata::NodeId,
    pub type_arguments: Option<crate::astdata::NodeList>,
}
#[derive(Debug, Default)]
pub struct NonNullExpressionData {
    pub value: i32,
    pub expression: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct OptionalTypeNodeData {
    pub value: i32,
    pub type_: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct ParameterDeclarationData {
    pub value: i32,
    pub name: crate::astdata::NodeId,
    pub question_token: Option<crate::astdata::NodeId>,
    pub type_: Option<crate::astdata::NodeId>,
    pub initializer: Option<crate::astdata::NodeId>,
    pub modifiers: Option<crate::astdata::ModifierList>,
}
#[derive(Debug, Default)]
pub struct ParenthesizedExpressionData {
    pub value: i32,
    pub expression: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct ParenthesizedTypeNodeData {
    pub value: i32,
    pub type_: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct PartiallyEmittedExpressionData {
    pub value: i32,
    pub expression: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct PropertyAccessExpressionData {
    pub value: i32,
    pub name: crate::astdata::NodeId,
    pub expression: crate::astdata::NodeId,
    pub question_dot_token: Option<crate::astdata::NodeId>,
}
#[derive(Debug, Default)]
pub struct PropertyAssignmentData {
    pub value: i32,
    pub name: crate::astdata::NodeId,
    pub postfix_token: Option<crate::astdata::NodeId>,
    pub type_: Option<crate::astdata::NodeId>,
    pub initializer: crate::astdata::NodeId,
    pub modifiers: Option<crate::astdata::ModifierList>,
}
#[derive(Debug, Default)]
pub struct PropertyDeclarationData {
    pub value: i32,
    pub name: crate::astdata::NodeId,
    pub postfix_token: Option<crate::astdata::NodeId>,
    pub type_: Option<crate::astdata::NodeId>,
    pub initializer: Option<crate::astdata::NodeId>,
    pub modifiers: Option<crate::astdata::ModifierList>,
}
#[derive(Debug, Default)]
pub struct PropertySignatureDeclarationData {
    pub value: i32,
    pub name: crate::astdata::NodeId,
    pub postfix_token: Option<crate::astdata::NodeId>,
    pub type_: crate::astdata::NodeId,
    pub initializer: crate::astdata::NodeId,
    pub modifiers: Option<crate::astdata::ModifierList>,
}
#[derive(Debug, Default)]
pub struct QualifiedNameData {
    pub value: i32,
    pub left: crate::astdata::NodeId,
    pub right: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct RestTypeNodeData {
    pub value: i32,
    pub type_: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct ReturnStatementData {
    pub value: i32,
    pub expression: Option<crate::astdata::NodeId>,
}
#[derive(Debug, Default)]
pub struct SatisfiesExpressionData {
    pub value: i32,
    pub expression: crate::astdata::NodeId,
    pub type_: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct SetAccessorDeclarationData {
    pub value: i32,
    pub name: crate::astdata::NodeId,
    pub postfix_token: Option<crate::astdata::NodeId>,
    pub type_: Option<crate::astdata::NodeId>,
    pub modifiers: Option<crate::astdata::ModifierList>,
    pub parameters: crate::astdata::NodeList,
}
#[derive(Debug, Default)]
pub struct ShorthandPropertyAssignmentData {
    pub value: i32,
    pub name: crate::astdata::NodeId,
    pub postfix_token: Option<crate::astdata::NodeId>,
    pub type_: Option<crate::astdata::NodeId>,
    pub modifiers: Option<crate::astdata::ModifierList>,
}
#[derive(Debug, Default)]
pub struct SourceFileData {
    pub value: i32,
}
#[derive(Debug, Default)]
pub struct SpreadAssignmentData {
    pub value: i32,
    pub expression: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct SpreadElementData {
    pub value: i32,
    pub expression: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct SwitchStatementData {
    pub value: i32,
    pub expression: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct SyntheticReferenceExpressionData {
    pub value: i32,
    pub expression: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct TaggedTemplateExpressionData {
    pub value: i32,
    pub type_arguments: Option<crate::astdata::NodeList>,
    pub question_dot_token: Option<crate::astdata::NodeId>,
}
#[derive(Debug, Default)]
pub struct TemplateLiteralTypeSpanData {
    pub value: i32,
    pub type_: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct TemplateSpanData {
    pub value: i32,
    pub expression: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct ThrowStatementData {
    pub value: i32,
    pub expression: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct TokenData {
    pub value: i32,
}
#[derive(Debug, Default)]
pub struct TypeAliasDeclarationData {
    pub value: i32,
    pub name: crate::astdata::NodeId,
    pub type_: crate::astdata::NodeId,
    pub modifiers: Option<crate::astdata::ModifierList>,
}
#[derive(Debug, Default)]
pub struct TypeAssertionData {
    pub value: i32,
    pub expression: crate::astdata::NodeId,
    pub type_: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct TypeOfExpressionData {
    pub value: i32,
    pub expression: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct TypeOperatorNodeData {
    pub value: i32,
    pub type_: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct TypeParameterDeclarationData {
    pub value: i32,
    pub name: crate::astdata::NodeId,
    pub expression: Option<crate::astdata::NodeId>,
    pub modifiers: Option<crate::astdata::ModifierList>,
}
#[derive(Debug, Default)]
pub struct TypePredicateNodeData {
    pub value: i32,
    pub type_: Option<crate::astdata::NodeId>,
}
#[derive(Debug, Default)]
pub struct TypeQueryNodeData {
    pub value: i32,
    pub type_arguments: Option<crate::astdata::NodeList>,
}
#[derive(Debug, Default)]
pub struct TypeReferenceNodeData {
    pub value: i32,
    pub type_arguments: Option<crate::astdata::NodeList>,
    pub type_name: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct VariableDeclarationData {
    pub value: i32,
    pub name: crate::astdata::NodeId,
    pub type_: Option<crate::astdata::NodeId>,
    pub initializer: Option<crate::astdata::NodeId>,
}
#[derive(Debug, Default)]
pub struct VariableStatementData {
    pub value: i32,
    pub modifiers: Option<crate::astdata::ModifierList>,
}
#[derive(Debug, Default)]
pub struct VoidExpressionData {
    pub value: i32,
    pub expression: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct WhileStatementData {
    pub value: i32,
    pub expression: crate::astdata::NodeId,
    pub statement: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct WithStatementData {
    pub value: i32,
    pub expression: crate::astdata::NodeId,
}
#[derive(Debug, Default)]
pub struct YieldExpressionData {
    pub value: i32,
    pub expression: Option<crate::astdata::NodeId>,
}

#[derive(Debug)]
pub enum NodeData {
    ArrowFunction(Box<ArrowFunctionData>),
    AsExpression(Box<AsExpressionData>),
    AwaitExpression(Box<AwaitExpressionData>),
    BinaryExpression(Box<BinaryExpressionData>),
    BindingElement(Box<BindingElementData>),
    Block(Box<BlockData>),
    CallExpression(Box<CallExpressionData>),
    CallSignatureDeclaration(Box<CallSignatureDeclarationData>),
    CaseBlock(Box<CaseBlockData>),
    CaseOrDefaultClause(Box<CaseOrDefaultClauseData>),
    CatchClause(Box<CatchClauseData>),
    ClassDeclaration(Box<ClassDeclarationData>),
    ClassExpression(Box<ClassExpressionData>),
    ClassStaticBlockDeclaration(Box<ClassStaticBlockDeclarationData>),
    ComputedPropertyName(Box<ComputedPropertyNameData>),
    ConditionalExpression(Box<ConditionalExpressionData>),
    ConditionalTypeNode(Box<ConditionalTypeNodeData>),
    ConstructSignatureDeclaration(Box<ConstructSignatureDeclarationData>),
    ConstructorDeclaration(Box<ConstructorDeclarationData>),
    ConstructorTypeNode(Box<ConstructorTypeNodeData>),
    Decorator(Box<DecoratorData>),
    DeleteExpression(Box<DeleteExpressionData>),
    DoStatement(Box<DoStatementData>),
    ElementAccessExpression(Box<ElementAccessExpressionData>),
    EnumDeclaration(Box<EnumDeclarationData>),
    EnumMember(Box<EnumMemberData>),
    ExportAssignment(Box<ExportAssignmentData>),
    ExportDeclaration(Box<ExportDeclarationData>),
    ExportSpecifier(Box<ExportSpecifierData>),
    ExpressionStatement(Box<ExpressionStatementData>),
    ExpressionWithTypeArguments(Box<ExpressionWithTypeArgumentsData>),
    ExternalModuleReference(Box<ExternalModuleReferenceData>),
    ForInOrOfStatement(Box<ForInOrOfStatementData>),
    ForStatement(Box<ForStatementData>),
    FunctionDeclaration(Box<FunctionDeclarationData>),
    FunctionExpression(Box<FunctionExpressionData>),
    FunctionTypeNode(Box<FunctionTypeNodeData>),
    GetAccessorDeclaration(Box<GetAccessorDeclarationData>),
    Identifier(Box<IdentifierData>),
    IfStatement(Box<IfStatementData>),
    ImportAttribute(Box<ImportAttributeData>),
    ImportClause(Box<ImportClauseData>),
    ImportDeclaration(Box<ImportDeclarationData>),
    ImportEqualsDeclaration(Box<ImportEqualsDeclarationData>),
    ImportSpecifier(Box<ImportSpecifierData>),
    ImportTypeNode(Box<ImportTypeNodeData>),
    IndexSignatureDeclaration(Box<IndexSignatureDeclarationData>),
    InterfaceDeclaration(Box<InterfaceDeclarationData>),
    JsDocCallbackTag(Box<JsDocCallbackTagData>),
    JsDocLink(Box<JsDocLinkData>),
    JsDocLinkCode(Box<JsDocLinkCodeData>),
    JsDocLinkPlain(Box<JsDocLinkPlainData>),
    JsDocNameReference(Box<JsDocNameReferenceData>),
    JsDocNonNullableType(Box<JsDocNonNullableTypeData>),
    JsDocNullableType(Box<JsDocNullableTypeData>),
    JsDocOptionalType(Box<JsDocOptionalTypeData>),
    JsDocParameterOrPropertyTag(Box<JsDocParameterOrPropertyTagData>),
    JsDocSignature(Box<JsDocSignatureData>),
    JsDocTypeExpression(Box<JsDocTypeExpressionData>),
    JsDocTypedefTag(Box<JsDocTypedefTagData>),
    JsDocVariadicType(Box<JsDocVariadicTypeData>),
    JsxAttribute(Box<JsxAttributeData>),
    JsxExpression(Box<JsxExpressionData>),
    JsxNamespacedName(Box<JsxNamespacedNameData>),
    JsxOpeningElement(Box<JsxOpeningElementData>),
    JsxSelfClosingElement(Box<JsxSelfClosingElementData>),
    JsxSpreadAttribute(Box<JsxSpreadAttributeData>),
    MappedTypeNode(Box<MappedTypeNodeData>),
    MetaProperty(Box<MetaPropertyData>),
    MethodDeclaration(Box<MethodDeclarationData>),
    MethodSignatureDeclaration(Box<MethodSignatureDeclarationData>),
    MissingDeclaration(Box<MissingDeclarationData>),
    ModuleDeclaration(Box<ModuleDeclarationData>),
    NamedTupleMember(Box<NamedTupleMemberData>),
    NamespaceExport(Box<NamespaceExportData>),
    NamespaceExportDeclaration(Box<NamespaceExportDeclarationData>),
    NamespaceImport(Box<NamespaceImportData>),
    NewExpression(Box<NewExpressionData>),
    NonNullExpression(Box<NonNullExpressionData>),
    OptionalTypeNode(Box<OptionalTypeNodeData>),
    ParameterDeclaration(Box<ParameterDeclarationData>),
    ParenthesizedExpression(Box<ParenthesizedExpressionData>),
    ParenthesizedTypeNode(Box<ParenthesizedTypeNodeData>),
    PartiallyEmittedExpression(Box<PartiallyEmittedExpressionData>),
    PropertyAccessExpression(Box<PropertyAccessExpressionData>),
    PropertyAssignment(Box<PropertyAssignmentData>),
    PropertyDeclaration(Box<PropertyDeclarationData>),
    PropertySignatureDeclaration(Box<PropertySignatureDeclarationData>),
    QualifiedName(Box<QualifiedNameData>),
    RestTypeNode(Box<RestTypeNodeData>),
    ReturnStatement(Box<ReturnStatementData>),
    SatisfiesExpression(Box<SatisfiesExpressionData>),
    SetAccessorDeclaration(Box<SetAccessorDeclarationData>),
    ShorthandPropertyAssignment(Box<ShorthandPropertyAssignmentData>),
    SourceFile(Box<SourceFileData>),
    SpreadAssignment(Box<SpreadAssignmentData>),
    SpreadElement(Box<SpreadElementData>),
    SwitchStatement(Box<SwitchStatementData>),
    SyntheticReferenceExpression(Box<SyntheticReferenceExpressionData>),
    TaggedTemplateExpression(Box<TaggedTemplateExpressionData>),
    TemplateLiteralTypeSpan(Box<TemplateLiteralTypeSpanData>),
    TemplateSpan(Box<TemplateSpanData>),
    ThrowStatement(Box<ThrowStatementData>),
    Token(Box<TokenData>),
    TypeAliasDeclaration(Box<TypeAliasDeclarationData>),
    TypeAssertion(Box<TypeAssertionData>),
    TypeOfExpression(Box<TypeOfExpressionData>),
    TypeOperatorNode(Box<TypeOperatorNodeData>),
    TypeParameterDeclaration(Box<TypeParameterDeclarationData>),
    TypePredicateNode(Box<TypePredicateNodeData>),
    TypeQueryNode(Box<TypeQueryNodeData>),
    TypeReferenceNode(Box<TypeReferenceNodeData>),
    VariableDeclaration(Box<VariableDeclarationData>),
    VariableStatement(Box<VariableStatementData>),
    VoidExpression(Box<VoidExpressionData>),
    WhileStatement(Box<WhileStatementData>),
    WithStatement(Box<WithStatementData>),
    YieldExpression(Box<YieldExpressionData>),
}
impl NodeData {
    pub fn schema_name(&self) -> &'static str {
        match self {
            NodeData::ArrowFunction(_) => "ArrowFunction",
            NodeData::AsExpression(_) => "AsExpression",
            NodeData::AwaitExpression(_) => "AwaitExpression",
            NodeData::BinaryExpression(_) => "BinaryExpression",
            NodeData::BindingElement(_) => "BindingElement",
            NodeData::Block(_) => "Block",
            NodeData::CallExpression(_) => "CallExpression",
            NodeData::CallSignatureDeclaration(_) => "CallSignatureDeclaration",
            NodeData::CaseBlock(_) => "CaseBlock",
            NodeData::CaseOrDefaultClause(_) => "CaseOrDefaultClause",
            NodeData::CatchClause(_) => "CatchClause",
            NodeData::ClassDeclaration(_) => "ClassDeclaration",
            NodeData::ClassExpression(_) => "ClassExpression",
            NodeData::ClassStaticBlockDeclaration(_) => "ClassStaticBlockDeclaration",
            NodeData::ComputedPropertyName(_) => "ComputedPropertyName",
            NodeData::ConditionalExpression(_) => "ConditionalExpression",
            NodeData::ConditionalTypeNode(_) => "ConditionalTypeNode",
            NodeData::ConstructSignatureDeclaration(_) => "ConstructSignatureDeclaration",
            NodeData::ConstructorDeclaration(_) => "ConstructorDeclaration",
            NodeData::ConstructorTypeNode(_) => "ConstructorTypeNode",
            NodeData::Decorator(_) => "Decorator",
            NodeData::DeleteExpression(_) => "DeleteExpression",
            NodeData::DoStatement(_) => "DoStatement",
            NodeData::ElementAccessExpression(_) => "ElementAccessExpression",
            NodeData::EnumDeclaration(_) => "EnumDeclaration",
            NodeData::EnumMember(_) => "EnumMember",
            NodeData::ExportAssignment(_) => "ExportAssignment",
            NodeData::ExportDeclaration(_) => "ExportDeclaration",
            NodeData::ExportSpecifier(_) => "ExportSpecifier",
            NodeData::ExpressionStatement(_) => "ExpressionStatement",
            NodeData::ExpressionWithTypeArguments(_) => "ExpressionWithTypeArguments",
            NodeData::ExternalModuleReference(_) => "ExternalModuleReference",
            NodeData::ForInOrOfStatement(_) => "ForInOrOfStatement",
            NodeData::ForStatement(_) => "ForStatement",
            NodeData::FunctionDeclaration(_) => "FunctionDeclaration",
            NodeData::FunctionExpression(_) => "FunctionExpression",
            NodeData::FunctionTypeNode(_) => "FunctionTypeNode",
            NodeData::GetAccessorDeclaration(_) => "GetAccessorDeclaration",
            NodeData::Identifier(_) => "Identifier",
            NodeData::IfStatement(_) => "IfStatement",
            NodeData::ImportAttribute(_) => "ImportAttribute",
            NodeData::ImportClause(_) => "ImportClause",
            NodeData::ImportDeclaration(_) => "ImportDeclaration",
            NodeData::ImportEqualsDeclaration(_) => "ImportEqualsDeclaration",
            NodeData::ImportSpecifier(_) => "ImportSpecifier",
            NodeData::ImportTypeNode(_) => "ImportTypeNode",
            NodeData::IndexSignatureDeclaration(_) => "IndexSignatureDeclaration",
            NodeData::InterfaceDeclaration(_) => "InterfaceDeclaration",
            NodeData::JsDocCallbackTag(_) => "JsDocCallbackTag",
            NodeData::JsDocLink(_) => "JsDocLink",
            NodeData::JsDocLinkCode(_) => "JsDocLinkCode",
            NodeData::JsDocLinkPlain(_) => "JsDocLinkPlain",
            NodeData::JsDocNameReference(_) => "JsDocNameReference",
            NodeData::JsDocNonNullableType(_) => "JsDocNonNullableType",
            NodeData::JsDocNullableType(_) => "JsDocNullableType",
            NodeData::JsDocOptionalType(_) => "JsDocOptionalType",
            NodeData::JsDocParameterOrPropertyTag(_) => "JsDocParameterOrPropertyTag",
            NodeData::JsDocSignature(_) => "JsDocSignature",
            NodeData::JsDocTypeExpression(_) => "JsDocTypeExpression",
            NodeData::JsDocTypedefTag(_) => "JsDocTypedefTag",
            NodeData::JsDocVariadicType(_) => "JsDocVariadicType",
            NodeData::JsxAttribute(_) => "JsxAttribute",
            NodeData::JsxExpression(_) => "JsxExpression",
            NodeData::JsxNamespacedName(_) => "JsxNamespacedName",
            NodeData::JsxOpeningElement(_) => "JsxOpeningElement",
            NodeData::JsxSelfClosingElement(_) => "JsxSelfClosingElement",
            NodeData::JsxSpreadAttribute(_) => "JsxSpreadAttribute",
            NodeData::MappedTypeNode(_) => "MappedTypeNode",
            NodeData::MetaProperty(_) => "MetaProperty",
            NodeData::MethodDeclaration(_) => "MethodDeclaration",
            NodeData::MethodSignatureDeclaration(_) => "MethodSignatureDeclaration",
            NodeData::MissingDeclaration(_) => "MissingDeclaration",
            NodeData::ModuleDeclaration(_) => "ModuleDeclaration",
            NodeData::NamedTupleMember(_) => "NamedTupleMember",
            NodeData::NamespaceExport(_) => "NamespaceExport",
            NodeData::NamespaceExportDeclaration(_) => "NamespaceExportDeclaration",
            NodeData::NamespaceImport(_) => "NamespaceImport",
            NodeData::NewExpression(_) => "NewExpression",
            NodeData::NonNullExpression(_) => "NonNullExpression",
            NodeData::OptionalTypeNode(_) => "OptionalTypeNode",
            NodeData::ParameterDeclaration(_) => "ParameterDeclaration",
            NodeData::ParenthesizedExpression(_) => "ParenthesizedExpression",
            NodeData::ParenthesizedTypeNode(_) => "ParenthesizedTypeNode",
            NodeData::PartiallyEmittedExpression(_) => "PartiallyEmittedExpression",
            NodeData::PropertyAccessExpression(_) => "PropertyAccessExpression",
            NodeData::PropertyAssignment(_) => "PropertyAssignment",
            NodeData::PropertyDeclaration(_) => "PropertyDeclaration",
            NodeData::PropertySignatureDeclaration(_) => "PropertySignatureDeclaration",
            NodeData::QualifiedName(_) => "QualifiedName",
            NodeData::RestTypeNode(_) => "RestTypeNode",
            NodeData::ReturnStatement(_) => "ReturnStatement",
            NodeData::SatisfiesExpression(_) => "SatisfiesExpression",
            NodeData::SetAccessorDeclaration(_) => "SetAccessorDeclaration",
            NodeData::ShorthandPropertyAssignment(_) => "ShorthandPropertyAssignment",
            NodeData::SourceFile(_) => "SourceFile",
            NodeData::SpreadAssignment(_) => "SpreadAssignment",
            NodeData::SpreadElement(_) => "SpreadElement",
            NodeData::SwitchStatement(_) => "SwitchStatement",
            NodeData::SyntheticReferenceExpression(_) => "SyntheticReferenceExpression",
            NodeData::TaggedTemplateExpression(_) => "TaggedTemplateExpression",
            NodeData::TemplateLiteralTypeSpan(_) => "TemplateLiteralTypeSpan",
            NodeData::TemplateSpan(_) => "TemplateSpan",
            NodeData::ThrowStatement(_) => "ThrowStatement",
            NodeData::Token(_) => "Token",
            NodeData::TypeAliasDeclaration(_) => "TypeAliasDeclaration",
            NodeData::TypeAssertion(_) => "TypeAssertion",
            NodeData::TypeOfExpression(_) => "TypeOfExpression",
            NodeData::TypeOperatorNode(_) => "TypeOperatorNode",
            NodeData::TypeParameterDeclaration(_) => "TypeParameterDeclaration",
            NodeData::TypePredicateNode(_) => "TypePredicateNode",
            NodeData::TypeQueryNode(_) => "TypeQueryNode",
            NodeData::TypeReferenceNode(_) => "TypeReferenceNode",
            NodeData::VariableDeclaration(_) => "VariableDeclaration",
            NodeData::VariableStatement(_) => "VariableStatement",
            NodeData::VoidExpression(_) => "VoidExpression",
            NodeData::WhileStatement(_) => "WhileStatement",
            NodeData::WithStatement(_) => "WithStatement",
            NodeData::YieldExpression(_) => "YieldExpression",
        }
    }
    pub fn kind(&self) -> SyntaxKind {
        match self {
            NodeData::ArrowFunction(_) => SyntaxKind::ArrowFunction,
            NodeData::AsExpression(_) => SyntaxKind::AsExpression,
            NodeData::AwaitExpression(_) => SyntaxKind::AwaitExpression,
            NodeData::BinaryExpression(_) => SyntaxKind::BinaryExpression,
            NodeData::BindingElement(_) => SyntaxKind::BindingElement,
            NodeData::Block(_) => SyntaxKind::Block,
            NodeData::CallExpression(_) => SyntaxKind::CallExpression,
            NodeData::CallSignatureDeclaration(_) => SyntaxKind::CallSignature,
            NodeData::CaseBlock(_) => SyntaxKind::CaseBlock,
            NodeData::CaseOrDefaultClause(_) => SyntaxKind::CaseOrDefaultClause,
            NodeData::CatchClause(_) => SyntaxKind::CatchClause,
            NodeData::ClassDeclaration(_) => SyntaxKind::ClassDeclaration,
            NodeData::ClassExpression(_) => SyntaxKind::ClassExpression,
            NodeData::ClassStaticBlockDeclaration(_) => SyntaxKind::ClassStaticBlockDeclaration,
            NodeData::ComputedPropertyName(_) => SyntaxKind::ComputedPropertyName,
            NodeData::ConditionalExpression(_) => SyntaxKind::ConditionalExpression,
            NodeData::ConditionalTypeNode(_) => SyntaxKind::ConditionalType,
            NodeData::ConstructSignatureDeclaration(_) => SyntaxKind::ConstructSignature,
            NodeData::ConstructorDeclaration(_) => SyntaxKind::Constructor,
            NodeData::ConstructorTypeNode(_) => SyntaxKind::ConstructorType,
            NodeData::Decorator(_) => SyntaxKind::Decorator,
            NodeData::DeleteExpression(_) => SyntaxKind::DeleteExpression,
            NodeData::DoStatement(_) => SyntaxKind::DoStatement,
            NodeData::ElementAccessExpression(_) => SyntaxKind::ElementAccessExpression,
            NodeData::EnumDeclaration(_) => SyntaxKind::EnumDeclaration,
            NodeData::EnumMember(_) => SyntaxKind::EnumMember,
            NodeData::ExportAssignment(_) => SyntaxKind::ExportAssignment,
            NodeData::ExportDeclaration(_) => SyntaxKind::ExportDeclaration,
            NodeData::ExportSpecifier(_) => SyntaxKind::ExportSpecifier,
            NodeData::ExpressionStatement(_) => SyntaxKind::ExpressionStatement,
            NodeData::ExpressionWithTypeArguments(_) => SyntaxKind::ExpressionWithTypeArguments,
            NodeData::ExternalModuleReference(_) => SyntaxKind::ExternalModuleReference,
            NodeData::ForInOrOfStatement(_) => SyntaxKind::ForInOrOfStatement,
            NodeData::ForStatement(_) => SyntaxKind::ForStatement,
            NodeData::FunctionDeclaration(_) => SyntaxKind::FunctionDeclaration,
            NodeData::FunctionExpression(_) => SyntaxKind::FunctionExpression,
            NodeData::FunctionTypeNode(_) => SyntaxKind::FunctionType,
            NodeData::GetAccessorDeclaration(_) => SyntaxKind::GetAccessor,
            NodeData::Identifier(_) => SyntaxKind::Identifier,
            NodeData::IfStatement(_) => SyntaxKind::IfStatement,
            NodeData::ImportAttribute(_) => SyntaxKind::ImportAttribute,
            NodeData::ImportClause(_) => SyntaxKind::ImportClause,
            NodeData::ImportDeclaration(_) => SyntaxKind::ImportDeclaration,
            NodeData::ImportEqualsDeclaration(_) => SyntaxKind::ImportEqualsDeclaration,
            NodeData::ImportSpecifier(_) => SyntaxKind::ImportSpecifier,
            NodeData::ImportTypeNode(_) => SyntaxKind::ImportType,
            NodeData::IndexSignatureDeclaration(_) => SyntaxKind::IndexSignature,
            NodeData::InterfaceDeclaration(_) => SyntaxKind::InterfaceDeclaration,
            NodeData::JsDocCallbackTag(_) => SyntaxKind::JsDocCallbackTag,
            NodeData::JsDocLink(_) => SyntaxKind::JsDocLink,
            NodeData::JsDocLinkCode(_) => SyntaxKind::JsDocLinkCode,
            NodeData::JsDocLinkPlain(_) => SyntaxKind::JsDocLinkPlain,
            NodeData::JsDocNameReference(_) => SyntaxKind::JsDocNameReference,
            NodeData::JsDocNonNullableType(_) => SyntaxKind::JsDocNonNullableType,
            NodeData::JsDocNullableType(_) => SyntaxKind::JsDocNullableType,
            NodeData::JsDocOptionalType(_) => SyntaxKind::JsDocOptionalType,
            NodeData::JsDocParameterOrPropertyTag(_) => SyntaxKind::JsDocParameterOrPropertyTag,
            NodeData::JsDocSignature(_) => SyntaxKind::JsDocSignature,
            NodeData::JsDocTypeExpression(_) => SyntaxKind::JsDocTypeExpression,
            NodeData::JsDocTypedefTag(_) => SyntaxKind::JsDocTypedefTag,
            NodeData::JsDocVariadicType(_) => SyntaxKind::JsDocVariadicType,
            NodeData::JsxAttribute(_) => SyntaxKind::JsxAttribute,
            NodeData::JsxExpression(_) => SyntaxKind::JsxExpression,
            NodeData::JsxNamespacedName(_) => SyntaxKind::JsxNamespacedName,
            NodeData::JsxOpeningElement(_) => SyntaxKind::JsxOpeningElement,
            NodeData::JsxSelfClosingElement(_) => SyntaxKind::JsxSelfClosingElement,
            NodeData::JsxSpreadAttribute(_) => SyntaxKind::JsxSpreadAttribute,
            NodeData::MappedTypeNode(_) => SyntaxKind::MappedType,
            NodeData::MetaProperty(_) => SyntaxKind::MetaProperty,
            NodeData::MethodDeclaration(_) => SyntaxKind::MethodDeclaration,
            NodeData::MethodSignatureDeclaration(_) => SyntaxKind::MethodSignature,
            NodeData::MissingDeclaration(_) => SyntaxKind::MissingDeclaration,
            NodeData::ModuleDeclaration(_) => SyntaxKind::ModuleDeclaration,
            NodeData::NamedTupleMember(_) => SyntaxKind::NamedTupleMember,
            NodeData::NamespaceExport(_) => SyntaxKind::NamespaceExport,
            NodeData::NamespaceExportDeclaration(_) => SyntaxKind::NamespaceExportDeclaration,
            NodeData::NamespaceImport(_) => SyntaxKind::NamespaceImport,
            NodeData::NewExpression(_) => SyntaxKind::NewExpression,
            NodeData::NonNullExpression(_) => SyntaxKind::NonNullExpression,
            NodeData::OptionalTypeNode(_) => SyntaxKind::OptionalType,
            NodeData::ParameterDeclaration(_) => SyntaxKind::Parameter,
            NodeData::ParenthesizedExpression(_) => SyntaxKind::ParenthesizedExpression,
            NodeData::ParenthesizedTypeNode(_) => SyntaxKind::ParenthesizedType,
            NodeData::PartiallyEmittedExpression(_) => SyntaxKind::PartiallyEmittedExpression,
            NodeData::PropertyAccessExpression(_) => SyntaxKind::PropertyAccessExpression,
            NodeData::PropertyAssignment(_) => SyntaxKind::PropertyAssignment,
            NodeData::PropertyDeclaration(_) => SyntaxKind::PropertyDeclaration,
            NodeData::PropertySignatureDeclaration(_) => SyntaxKind::PropertySignature,
            NodeData::QualifiedName(_) => SyntaxKind::QualifiedName,
            NodeData::RestTypeNode(_) => SyntaxKind::RestType,
            NodeData::ReturnStatement(_) => SyntaxKind::ReturnStatement,
            NodeData::SatisfiesExpression(_) => SyntaxKind::SatisfiesExpression,
            NodeData::SetAccessorDeclaration(_) => SyntaxKind::SetAccessor,
            NodeData::ShorthandPropertyAssignment(_) => SyntaxKind::ShorthandPropertyAssignment,
            NodeData::SourceFile(_) => SyntaxKind::SourceFile,
            NodeData::SpreadAssignment(_) => SyntaxKind::SpreadAssignment,
            NodeData::SpreadElement(_) => SyntaxKind::SpreadElement,
            NodeData::SwitchStatement(_) => SyntaxKind::SwitchStatement,
            NodeData::SyntheticReferenceExpression(_) => SyntaxKind::SyntheticReferenceExpression,
            NodeData::TaggedTemplateExpression(_) => SyntaxKind::TaggedTemplateExpression,
            NodeData::TemplateLiteralTypeSpan(_) => SyntaxKind::TemplateLiteralTypeSpan,
            NodeData::TemplateSpan(_) => SyntaxKind::TemplateSpan,
            NodeData::ThrowStatement(_) => SyntaxKind::ThrowStatement,
            NodeData::Token(_) => SyntaxKind::Token,
            NodeData::TypeAliasDeclaration(_) => SyntaxKind::TypeAliasDeclaration,
            NodeData::TypeAssertion(_) => SyntaxKind::TypeAssertionExpression,
            NodeData::TypeOfExpression(_) => SyntaxKind::TypeOfExpression,
            NodeData::TypeOperatorNode(_) => SyntaxKind::TypeOperator,
            NodeData::TypeParameterDeclaration(_) => SyntaxKind::TypeParameter,
            NodeData::TypePredicateNode(_) => SyntaxKind::TypePredicate,
            NodeData::TypeQueryNode(_) => SyntaxKind::TypeQuery,
            NodeData::TypeReferenceNode(_) => SyntaxKind::TypeReference,
            NodeData::VariableDeclaration(_) => SyntaxKind::VariableDeclaration,
            NodeData::VariableStatement(_) => SyntaxKind::VariableStatement,
            NodeData::VoidExpression(_) => SyntaxKind::VoidExpression,
            NodeData::WhileStatement(_) => SyntaxKind::WhileStatement,
            NodeData::WithStatement(_) => SyntaxKind::WithStatement,
            NodeData::YieldExpression(_) => SyntaxKind::YieldExpression,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Node { file: u32, id: u32 }
impl Node {
    pub const NIL: Node = Node { file: 0, id: 0 };
    pub fn new(file: usize, id: crate::astdata::NodeId) -> Node { Node { file: file as u32, id: id.0 } }
    pub fn file_index(self) -> usize { self.file as usize }
    pub fn raw(self) -> u32 { self.id }
    pub fn is_nil(self) -> bool { self.id == 0 }
    pub fn kind(self) -> SyntaxKind {
        if self.is_nil() { return SyntaxKind::Unknown; }
        static_ast_node(self).map_or(SyntaxKind::Unknown, NodeData::kind)
    }
}

fn registry() -> &'static std::sync::Mutex<Vec<&'static NodeData>> {
    static REG: std::sync::OnceLock<std::sync::Mutex<Vec<&'static NodeData>>> = std::sync::OnceLock::new();
    REG.get_or_init(|| {
        let nil: &'static NodeData = Box::leak(Box::new(NodeData::Identifier(Box::default())));
        std::sync::Mutex::new(vec![nil])
    })
}
pub fn register(file: usize, data: NodeData) -> Node {
    let leaked: &'static NodeData = Box::leak(Box::new(data));
    let mut reg = registry().lock().unwrap();
    let id = reg.len() as u32;
    reg.push(leaked);
    Node { file: file as u32, id }
}
pub fn static_ast_node(n: Node) -> Option<&'static NodeData> {
    registry().lock().unwrap().get(n.id as usize).copied()
}
pub fn frozen_store_kind(n: Node) -> Option<SyntaxKind> {
    if n.is_nil() { return Some(SyntaxKind::Unknown); }
    static_ast_node(n).map(NodeData::kind)
}
pub fn is_data_of(d: &NodeData, n: Node) -> bool {
    static_ast_node(n).is_some_and(|data| std::ptr::eq(d, data))
}
pub fn opt(file: usize, id: Option<crate::astdata::NodeId>) -> Node {
    match id { Some(id) => Node::new(file, id), None => Node::NIL }
}
pub fn req(file: usize, id: crate::astdata::NodeId) -> Node { Node::new(file, id) }
pub fn some_opt(file: usize, id: Option<crate::astdata::NodeId>) -> Option<Node> { id.map(|i| req(file, i)) }
pub fn some_req(file: usize, id: crate::astdata::NodeId) -> Option<Node> { Some(req(file, id)) }
pub fn mods(file: usize, m: &'static Option<crate::astdata::ModifierList>) -> ModifierList {
    ModifierList::from_ts(file, m.as_ref())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeList { file: u32, list: Option<&'static crate::astdata::NodeList> }
impl NodeList {
    pub const NIL: NodeList = NodeList { file: 0, list: None };
    pub fn from_ts(file: usize, list: Option<&'static crate::astdata::NodeList>) -> Self {
        match list { Some(list) => NodeList { file: file as u32, list: Some(list) }, None => Self::NIL }
    }
    pub fn len(self) -> usize { self.list.map_or(0, |l| l.items.len()) }
    pub fn is_nil(self) -> bool { self.list.is_none() }
    pub fn nodes(self) -> Vec<Node> {
        self.list.map_or_else(Vec::new, |l| l.items.iter().map(|i| Node::new(self.file as usize, *i)).collect())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModifierList { file: u32, list: Option<&'static crate::astdata::ModifierList> }
impl ModifierList {
    pub const NIL: ModifierList = ModifierList { file: 0, list: None };
    pub fn from_ts(file: usize, list: Option<&'static crate::astdata::ModifierList>) -> Self {
        match list { Some(list) => ModifierList { file: file as u32, list: Some(list) }, None => Self::NIL }
    }
    pub fn bits(self) -> u32 { self.list.map_or(0, |l| l.flags.0) }
}

#[derive(Clone, Copy)]
pub struct DataList<'a> { pub file: usize, pub list: Option<&'a crate::astdata::NodeList> }
impl<'a> DataList<'a> {
    pub fn req(file: usize, l: &'a crate::astdata::NodeList) -> Self { DataList { file, list: Some(l) } }
    pub fn opt(file: usize, l: &'a Option<crate::astdata::NodeList>) -> Self { DataList { file, list: l.as_ref() } }
    pub fn mods(file: usize, m: &'a Option<crate::astdata::ModifierList>) -> Self {
        DataList { file, list: m.as_ref().map(|m| &m.list) }
    }
    pub fn len(self) -> usize { self.list.map_or(0, |l| l.items.len()) }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct SlotChildren { pub name: Option<u32>, pub other: Option<(u32, u32)>, pub typed: Option<(u32, u32)>, pub no_type_arguments: bool }
impl SlotChildren {
    pub const TAG_EXPRESSION: u32 = 1;
    pub const TAG_POSTFIX: u32 = 2;
    pub const TAG_QUESTION: u32 = 3;
    pub const TYPED_TYPE: u32 = 4;
    pub const TYPED_INITIALIZER: u32 = 5;
    pub fn new(name: Option<u32>, other: Option<(u32, u32)>) -> Self { SlotChildren { name, other, typed: None, no_type_arguments: false } }
    pub fn with_typed(mut self, typed: Option<(u32, u32)>, no_type_arguments: bool) -> Self {
        self.typed = typed; self.no_type_arguments = no_type_arguments; self
    }
}
// ---------- end scaffolding ----------
