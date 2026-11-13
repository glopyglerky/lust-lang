//! Port of Go `ast.NodeFactory` (`ast/ast.go`, `ast/ast_generated.go`).
//!
//! Every `New*` constructor that the checker, the node builder and the printer
//! use. A factory allocates in the synthetic arena (`ast/synthetic.rs`), or,
//! when made with `NodeFactory::for_file`, in the node store of one parsed
//! file (`ast/store.rs`).
//!
//! Argument rules:
//! - Go `*Node` is `Node`; Go `nil` is `Node::NIL`.
//! - Go `*NodeList` is `NodeList`; Go `*ModifierList` is `ModifierList`.
//!   `NodeList::NIL` / `ModifierList::NIL` is Go `nil`.
//! - A node of another file (or a parsed node for a synthetic target) may be
//!   passed as a child. The new node then refers to that same node (Go
//!   shares the pointer) through an alias slot; see `synthetic.rs` and
//!   `store.rs`.
//!
//! Go `newNode` does not set parents. Callers set `Parent` when Go does, with
//! `set_node_parent`.

use crate::astdata::NodeData as D;
use crate::prelude::*;

/// Go `*ast.NodeFactory`.
// PORT: Go `NodeFactoryHooks` (OnCreate/OnUpdate/OnClone) live in
// `ast/update.rs`; only the printer's emit context sets them. The counters use
// `Cell` so a shared `&NodeFactory` can create nodes while the checker is
// borrowed.
#[derive(Debug, Default)]
pub struct NodeFactory {
    hooks: NodeFactoryHooks,
    node_count: std::cell::Cell<usize>,
    text_count: std::cell::Cell<usize>,
    target: NodeFactoryTarget,
}

/// Where a `NodeFactory` puts new nodes.
// PORT: Go allocates every node on the heap. Here the parser's factory
// writes into the store of the file it parses, and every other factory into
// the synthetic arena.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NodeFactoryTarget {
    /// The synthetic arena (checker, node builder, printer, transforms).
    #[default]
    Synthetic,
    /// The node store with this id (the ported parser).
    File(usize),
}

/// ts_ast token flags from Go token flags.
fn token_flags(flags: TokenFlags) -> crate::astdata::TokenFlags {
    crate::astdata::TokenFlags(flags.bits() as u32)
}

const NO_TOKEN_FLAGS: crate::astdata::TokenFlags = crate::astdata::TokenFlags(0);

impl NodeFactory {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The parser's factory: new nodes and lists go into node store `store`
    /// (`new_file_store`).
    #[must_use]
    pub fn for_file(store: usize) -> Self {
        Self {
            target: NodeFactoryTarget::File(store),
            ..Self::default()
        }
    }

    /// Where this factory puts new nodes.
    #[must_use]
    pub fn target(&self) -> NodeFactoryTarget {
        self.target
    }






    // Go: ast/ast.go:71 NewNodeFactory
    #[must_use]
    pub fn new_with_hooks(hooks: NodeFactoryHooks) -> Self {
        Self {
            hooks,
            ..Self::default()
        }
    }

    /// Go `f.hooks`.
    pub(crate) fn hooks(&self) -> &NodeFactoryHooks {
        &self.hooks
    }

    // Go: ast/ast.go:84 (f *NodeFactory) newNode




    // Go: ast/ast.go:91 NodeCount
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.node_count.get()
    }

    // Go: ast/ast.go:95 TextCount
    #[must_use]
    pub fn text_count(&self) -> usize {
        self.text_count.get()
    }

    // ── Source files ───────────────────────────────────────────────────

    // Go: ast/ast.go:2526 NewSourceFile
    // PORT: Go takes `opts SourceFileParseOptions`; only its `FileName` and
    // `Path` are kept (see `SyntheticSourceFileData`). A synthetic
    // SourceFile keeps the Go fields in its slot. A store SourceFile (the
    // ported parser) keeps file name and text in its node store.


    // ── Lists ──────────────────────────────────────────────────────────

    // Go: ast/ast.go:129 NewNodeList


    // Go: ast/ast.go:160 NewModifierList


    // ── Tokens, names and literals ─────────────────────────────────────

    // Go: ast/ast_generated.go:600 NewToken

    // Go: ast/ast.go:1674 NewModifier

    // Go: ast/ast_generated.go:627 NewIdentifier

    // Go: ast/ast_generated.go:651 NewPrivateIdentifier

    // Go: ast/ast_generated.go:678 NewQualifiedName

    // Go: ast/ast_generated.go:723 NewComputedPropertyName

    // Go: ast/ast_generated.go:3601 NewStringLiteral

    // Go: ast/ast_generated.go:3625 NewNumericLiteral

    // Go: ast/ast_generated.go:3649 NewBigIntLiteral

    // Go: ast/ast_generated.go:3673 NewRegularExpressionLiteral

    // Go: ast/ast_generated.go:3699 NewNoSubstitutionTemplateLiteral
    // PORT: Go sets no `RawText`; ts_ast stores an empty string for it.

    // Go: ast/ast_generated.go:6111 NewTemplateHead

    // Go: ast/ast_generated.go:6137 NewTemplateMiddle

    // Go: ast/ast_generated.go:6163 NewTemplateTail

    // ── Type nodes ─────────────────────────────────────────────────────

    // Go: ast/ast_generated.go:5110 NewKeywordTypeNode

    // Go: ast/ast_generated.go:5135 NewUnionTypeNode

    // Go: ast/ast_generated.go:5172 NewIntersectionTypeNode

    // Go: ast/ast_generated.go:5214 NewConditionalTypeNode

    // Go: ast/ast_generated.go:5259 NewTypeOperatorNode

    // Go: ast/ast_generated.go:5298 NewInferTypeNode

    // Go: ast/ast_generated.go:5336 NewArrayTypeNode

    // Go: ast/ast_generated.go:5375 NewIndexedAccessTypeNode

    // Go: ast/ast_generated.go:5414 NewTypeReferenceNode

    // Go: ast/ast_generated.go:5455 NewExpressionWithTypeArguments

    // Go: ast/ast_generated.go:5494 NewLiteralTypeNode

    // Go: ast/ast_generated.go:5531 NewThisTypeNode

    // Go: ast/ast_generated.go:5555 NewTypePredicateNode

    // Go: ast/ast_generated.go:5692 NewTypeQueryNode

    // Go: ast/ast_generated.go:5738 NewMappedTypeNode

    // Go: ast/ast_generated.go:5787 NewTypeLiteralNode

    // Go: ast/ast_generated.go:5825 NewTupleTypeNode

    // Go: ast/ast_generated.go:5867 NewNamedTupleMember

    // Go: ast/ast_generated.go:5915 NewOptionalTypeNode

    // Go: ast/ast_generated.go:5953 NewRestTypeNode

    // Go: ast/ast_generated.go:5991 NewParenthesizedTypeNode

    // Go: ast/ast_generated.go:6028 NewFunctionTypeNode

    // Go: ast/ast_generated.go:6067 NewConstructorTypeNode

    // Go: ast/ast_generated.go:6190 NewTemplateLiteralTypeNode

    // Go: ast/ast_generated.go:6230 NewTemplateLiteralTypeSpan

    // Go: ast/ast_generated.go:8203 NewImportTypeNode

    // ── Signatures and type elements ───────────────────────────────────

    // Go: ast/ast_generated.go:8507 NewTypeParameterDeclaration

    // Go: ast/ast_generated.go:1848 NewParameterDeclaration

    // Go: ast/ast_generated.go:2970 NewCallSignatureDeclaration

    // Go: ast/ast_generated.go:3013 NewConstructSignatureDeclaration

    // Go: ast/ast_generated.go:3058 NewConstructorDeclaration

    // Go: ast/ast_generated.go:3105 NewGetAccessorDeclaration

    // Go: ast/ast_generated.go:3158 NewSetAccessorDeclaration

    // Go: ast/ast_generated.go:3217 NewIndexSignatureDeclaration
    // PORT: Go keeps a nil `Type`; ts_ast requires one, so nil is stored in
    // the nil slot and `type_node()` still reads nil.

    // Go: ast/ast_generated.go:3261 NewMethodSignatureDeclaration

    // Go: ast/ast_generated.go:3319 NewMethodDeclaration

    // Go: ast/ast_generated.go:3382 NewPropertySignatureDeclaration
    // PORT: ts_ast requires `Type` and `Initializer`; Go nil goes to the nil
    // slot, so `type_node()` and `initializer()` still read nil.

    // Go: ast/ast_generated.go:3437 NewPropertyDeclaration

    // Go: ast/ast_generated.go:2498 NewNotEmittedTypeElement

    // ── Declarations ───────────────────────────────────────────────────

    // Go: ast/ast_generated.go:2341 NewEnumMember

    // Go: ast/ast_generated.go:2389 NewEnumDeclaration

    // Go: ast/ast_generated.go:2000 NewFunctionDeclaration

    // Go: ast/ast_generated.go:2057 NewClassDeclaration

    // Go: ast/ast_generated.go:2108 NewClassExpression

    // Go: ast/ast_generated.go:2160 NewHeritageClause

    // Go: ast/ast_generated.go:2206 NewInterfaceDeclaration

    // Go: ast/ast_generated.go:2263 NewTypeAliasDeclaration

    // Go: ast/ast_generated.go:8049 NewModuleDeclaration

    // Go: ast/ast_generated.go:2434 NewModuleBlock

    // Go: ast/ast_generated.go:5597 NewImportAttribute

    // Go: ast/ast_generated.go:5648 NewImportAttributes

    // Go: ast/ast_generated.go:2741 NewExportAssignment

    // Go: ast/ast_generated.go:2876 NewNamedExports

    // Go: ast/ast_generated.go:2923 NewExportSpecifier

    // Go: ast/ast_generated.go:8155 NewExportDeclaration

    // Go: ast/ast_generated.go:2525 NewImportDeclaration

    // Go: ast/ast_generated.go:2603 NewExternalModuleReference

    // Go: ast/ast_generated.go:2694 NewNamedImports

    // Go: ast/ast_generated.go:6842 NewSyntaxList

    // Go: ast/ast_generated.go:8104 NewImportEqualsDeclaration

    // Go: ast/ast_generated.go:8253 NewImportClause
    // PORT: Go KindUnknown for `phaseModifier` is `None` in the data.

    // Go: ast/ast_generated.go:8302 NewImportSpecifier

    // ── Expressions ────────────────────────────────────────────────────

    // Go: ast/ast_generated.go:3576 NewKeywordExpression

    // Go: ast/ast_generated.go:4164 NewPropertyAccessExpression

    // Go: ast/ast_generated.go:4214 NewElementAccessExpression

    // Go: ast/ast_generated.go:4267 NewCallExpression

    // Go: ast/ast_generated.go:4316 NewNewExpression

    // Go: ast/ast_generated.go:4402 NewNonNullExpression

    // Go: ast/ast_generated.go:4575 NewTaggedTemplateExpression

    // Go: ast/ast_generated.go:3777 NewPrefixUnaryExpression

    // Go: ast/ast_generated.go:3730 NewBinaryExpression

    // Go: ast/ast_generated.go:4106 NewConditionalExpression

    // Go: ast/ast_generated.go:4621 NewParenthesizedExpression

    // Go: ast/ast_generated.go:3965 NewFunctionExpression

    // Go: ast/ast_generated.go:3909 NewArrowFunction

    // Go: ast/ast_generated.go:6271 NewSyntheticExpression
    // PORT: Go `Type any` holds a `*checker.Type`; here it is a `TypeId`,
    // read back with `synthetic_expression_type`.

    // Go: ast/ast_generated.go:4442 NewSpreadElement

    // Go: ast/ast_generated.go:4665 NewArrayLiteralExpression

    // Go: ast/ast_generated.go:4711 NewObjectLiteralExpression

    // Go: ast/ast_generated.go:4799 NewPropertyAssignment

    // Go: ast/ast_generated.go:4022 NewAsExpression

    // Go: ast/ast_generated.go:4062 NewSatisfiesExpression

    // Go: ast/ast_generated.go:5072 NewTypeAssertion

    // Go: ast/ast_generated.go:4907 NewDeleteExpression

    // Go: ast/ast_generated.go:4949 NewTypeOfExpression

    // Go: ast/ast_generated.go:4991 NewVoidExpression

    // Go: ast/ast_generated.go:5033 NewAwaitExpression

    // Go: ast/ast_generated.go:3865 NewYieldExpression

    // Go: ast/ast_generated.go:766 NewDecorator

    // ── Statements ─────────────────────────────────────────────────────

    // Go: ast/ast_generated.go:1619 NewBlock

    // Go: ast/ast_generated.go:1664 NewVariableStatement

    // Go: ast/ast_generated.go:1709 NewVariableDeclaration

    // Go: ast/ast_generated.go:1758 NewVariableDeclarationList

    // Go: ast/ast_generated.go:1574 NewExpressionStatement

    // Go: ast/ast_generated.go:1149 NewReturnStatement

    // Go: ast/ast_generated.go:828 NewIfStatement

    // Go: ast/ast_generated.go:803 NewEmptyStatement

    // Go: ast/ast_generated.go:2475 NewNotEmittedStatement
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_nodes_read_like_go_nodes() {
        let f = NodeFactory::new();
        let this = f.new_keyword_expression(SyntaxKind::ThisKeyword);
        let name = f.new_identifier("x");
        let access = f.new_property_access_expression(
            this,
            Node::NIL,
            name,
            NodeFlags::OPTIONAL_CHAIN | NodeFlags::SYNTHESIZED,
        );

        // Go `newNode`: undefined loc, nil parent, and only OptionalChain kept.
        assert_eq!(access.kind(), SyntaxKind::PropertyAccessExpression);
        assert_eq!(access.flags(), NodeFlags::OPTIONAL_CHAIN);
        assert_eq!(access.loc(), TextRange::undefined());
        assert!(access.parent().is_nil());
        // Children keep their identity, like Go pointers.
        assert_eq!(access.expression(), this);
        assert_eq!(access.name(), name);
        assert_eq!(name.text(), "x");

        // Go field writes after creation.
        set_node_parent(this, access);
        assert_eq!(this.parent(), access);
        set_node_loc(access, TextRange::new(3, 7));
        assert_eq!(access.pos(), 3);

        // Go nil in a field that ts_ast requires still reads as nil.
        let sig =
            f.new_index_signature_declaration(ModifierList::NIL, f.new_node_list(&[]), Node::NIL);
        assert!(sig.type_().is_nil());
        assert_eq!(f.node_count(), 4);
        assert_eq!(f.text_count(), 1);
    }

    #[test]
    fn literal_flags_are_masked_like_go() {
        let f = NodeFactory::new();
        let s = f.new_string_literal("a", TokenFlags(-1));
        assert_eq!(s.token_flags(), TokenFlags::STRING_LITERAL_FLAGS);
        let list = f.new_node_list(&[s]);
        assert_eq!(list.loc(), TextRange::undefined());
        assert_eq!(list.nodes().get(0), s);
    }

    #[test]
    fn synthetic_expression_keeps_its_type() {
        let f = NodeFactory::new();
        let e = f.new_synthetic_expression(TypeId(42), true, Node::NIL);
        assert_eq!(synthetic_expression_type(e), TypeId(42));
        assert!(e.is_spread());
        assert!(e.tuple_name_source().is_nil());
    }
}
