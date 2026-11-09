//! Go `*ast.Node` methods, `NodeList`, `ModifierList`, `core.TextRange`,
//! subtree facts, access kinds and the `ast.SourceFile` accessors from pinned
//! typescript-go `internal/ast/ast.go` (and `internal/core/text.go` for
//! `TextRange`).
//!
//! Go reads the AST without a context. So do we: every method reaches the
//! parsed astdata arena through the file registry (`ast/store.rs`
//! `go_file`). Nodes of a ported-parser file are
//! read from its node store (`ast/store.rs`), and factory nodes from the
//! synthetic arena (`ast/synthetic.rs`).
//!
//! PORT: Go methods that panic with "Unhandled case in Node.X" return nil,
//! an empty list or "" here (PORTING.md "AST"). They also cover every node
//! struct that has the Go field of the same name, because `fields.rs` does not
//! generate accessors for names that clash with Go `Node` methods.
//!
//! PORT: the factories (`New*`), `Clone`, `VisitEachChild`, the printer, the
//! `MutableNode` setters and the `AsFlow*` casts are out of scope.

use crate::astdata::NodeData;
use crate::prelude::*;
use std::sync::atomic::Ordering;

// ──────────────────────────────────────────────────────────────────────
// Arena helpers
// ──────────────────────────────────────────────────────────────────────

/// The astdata id of `n`.
// PORT: `Node::node_id` in core.rs passes a usize to `NodeId::new(u32)`, so
// this file computes the id itself.
fn nid(n: Node) -> crate::astdata::NodeId {
    crate::astdata::NodeId::new(((n.0 & 0xffff_ffff) - 1) as u32)
}

/// The selector result for a list field (see `list_of!`): `req` for a
/// required list, `opt` for an optional one, `mods` for an optional modifier
/// list.
macro_rules! sel_field {
    (req, $x:expr) => {
        Some(AnyList::Nodes(&$x))
    };
    (opt, $x:expr) => {
        $x.as_ref().map(AnyList::Nodes)
    };
    (mods, $x:expr) => {
        $x.as_ref().map(AnyList::Modifiers)
    };
}






/// A list read in place from the data of a node of `file`, parsed or
/// synthetic. `None` is Go `nil`. Code that walks the lists of node data it
/// reads in a scope (subtree facts, the child walk of a synthetic node)
/// reads them this way; accessors return `NodeList` handles instead.
#[derive(Clone, Copy)]
struct DataList<'a> {
    file: usize,
    list: Option<&'a crate::astdata::NodeList>,
}

impl<'a> DataList<'a> {
    /// A required list.
    fn req(file: usize, l: &'a crate::astdata::NodeList) -> Self {
        Self {
            file,
            list: Some(l),
        }
    }

    /// An optional list.
    fn opt(file: usize, l: &'a Option<crate::astdata::NodeList>) -> Self {
        Self {
            file,
            list: l.as_ref(),
        }
    }

    /// The list of an optional modifier list.
    fn mods(file: usize, m: &'a Option<crate::astdata::ModifierList>) -> Self {
        Self {
            file,
            list: m.as_ref().map(|m| &m.list),
        }
    }

    /// Go `list != nil` (see `NodeList::is_nil`).
    fn is_some(self) -> bool {
        self.list.is_some_and(|l| !is_nil_list_marker(l))
    }

}

/// `ModifierList::modifier_flags` of a modifier list read in place from
/// node data (store or synthetic).
fn in_place_modifier_flags(m: &Option<crate::astdata::ModifierList>) -> ModifierFlags {
    m.as_ref()
        .map_or(ModifierFlags::NONE, |m| ModifierFlags(m.flags.0))
}

/// True when data variant `$v` fits a node of kind `$k`. This mirrors
/// `crate::astdata::NodeData::matches_syntax_kind`: a variant named like a kind fits
/// that kind, and the other variants are listed here. A variant that is not
/// listed and is not named like a kind does not compile.
macro_rules! variant_has_kind {
    (Token, $k:expr) => {
        $k.is_token()
    };
    (KeywordExpression, $k:expr) => {
        $k.is_keyword_expression()
    };
    (KeywordTypeNode, $k:expr) => {
        $k.is_keyword_type()
    };
    (ForInOrOfStatement, $k:expr) => {
        matches!($k, SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement)
    };
    (CaseOrDefaultClause, $k:expr) => {
        matches!($k, SyntaxKind::CaseClause | SyntaxKind::DefaultClause)
    };
    (BindingPattern, $k:expr) => {
        matches!(
            $k,
            SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern
        )
    };
    (JsDocParameterOrPropertyTag, $k:expr) => {
        matches!(
            $k,
            SyntaxKind::JsDocParameterTag | SyntaxKind::JsDocPropertyTag
        )
    };
    // These three are named like a kind but also fit a second kind.
    (TypeAliasDeclaration, $k:expr) => {
        matches!(
            $k,
            SyntaxKind::TypeAliasDeclaration | SyntaxKind::JsTypeAliasDeclaration
        )
    };
    (ImportDeclaration, $k:expr) => {
        matches!(
            $k,
            SyntaxKind::ImportDeclaration | SyntaxKind::JsImportDeclaration
        )
    };
    (JsxText, $k:expr) => {
        matches!($k, SyntaxKind::JsxText | SyntaxKind::JsxTextAllWhiteSpaces)
    };
    (ParameterDeclaration, $k:expr) => {
        $k == SyntaxKind::Parameter
    };
    (CallSignatureDeclaration, $k:expr) => {
        $k == SyntaxKind::CallSignature
    };
    (ConstructSignatureDeclaration, $k:expr) => {
        $k == SyntaxKind::ConstructSignature
    };
    (ConstructorDeclaration, $k:expr) => {
        $k == SyntaxKind::Constructor
    };
    (GetAccessorDeclaration, $k:expr) => {
        $k == SyntaxKind::GetAccessor
    };
    (SetAccessorDeclaration, $k:expr) => {
        $k == SyntaxKind::SetAccessor
    };
    (IndexSignatureDeclaration, $k:expr) => {
        $k == SyntaxKind::IndexSignature
    };
    (MethodSignatureDeclaration, $k:expr) => {
        $k == SyntaxKind::MethodSignature
    };
    (PropertySignatureDeclaration, $k:expr) => {
        $k == SyntaxKind::PropertySignature
    };
    (TypeParameterDeclaration, $k:expr) => {
        $k == SyntaxKind::TypeParameter
    };
    (TypeAssertion, $k:expr) => {
        $k == SyntaxKind::TypeAssertionExpression
    };
    (UnionTypeNode, $k:expr) => {
        $k == SyntaxKind::UnionType
    };
    (IntersectionTypeNode, $k:expr) => {
        $k == SyntaxKind::IntersectionType
    };
    (ConditionalTypeNode, $k:expr) => {
        $k == SyntaxKind::ConditionalType
    };
    (TypeOperatorNode, $k:expr) => {
        $k == SyntaxKind::TypeOperator
    };
    (InferTypeNode, $k:expr) => {
        $k == SyntaxKind::InferType
    };
    (ArrayTypeNode, $k:expr) => {
        $k == SyntaxKind::ArrayType
    };
    (IndexedAccessTypeNode, $k:expr) => {
        $k == SyntaxKind::IndexedAccessType
    };
    (TypeReferenceNode, $k:expr) => {
        $k == SyntaxKind::TypeReference
    };
    (LiteralTypeNode, $k:expr) => {
        $k == SyntaxKind::LiteralType
    };
    (ThisTypeNode, $k:expr) => {
        $k == SyntaxKind::ThisType
    };
    (TypePredicateNode, $k:expr) => {
        $k == SyntaxKind::TypePredicate
    };
    (TypeQueryNode, $k:expr) => {
        $k == SyntaxKind::TypeQuery
    };
    (MappedTypeNode, $k:expr) => {
        $k == SyntaxKind::MappedType
    };
    (TypeLiteralNode, $k:expr) => {
        $k == SyntaxKind::TypeLiteral
    };
    (TupleTypeNode, $k:expr) => {
        $k == SyntaxKind::TupleType
    };
    (OptionalTypeNode, $k:expr) => {
        $k == SyntaxKind::OptionalType
    };
    (RestTypeNode, $k:expr) => {
        $k == SyntaxKind::RestType
    };
    (ParenthesizedTypeNode, $k:expr) => {
        $k == SyntaxKind::ParenthesizedType
    };
    (FunctionTypeNode, $k:expr) => {
        $k == SyntaxKind::FunctionType
    };
    (ConstructorTypeNode, $k:expr) => {
        $k == SyntaxKind::ConstructorType
    };
    (TemplateLiteralTypeNode, $k:expr) => {
        $k == SyntaxKind::TemplateLiteralType
    };
    (ImportTypeNode, $k:expr) => {
        $k == SyntaxKind::ImportType
    };
    ($v:ident, $k:expr) => {
        $k == SyntaxKind::$v
    };
}

/// True when `n` is a published store node whose kind fits none of the listed
/// data variants. Such a node cannot hold the field, so the accessor gives
/// its default without loading the node data.
// Store data always fits the header kind (`alloc_store_node`). `Unknown` is
// also the kind of nil and alias slots, so it takes the data path, which
// keeps its panics.
macro_rules! kind_lacks_data {
    ($n:expr, [$($v:ident),+]) => {
        match frozen_store_kind($n) {
            Some(kind) => kind != SyntaxKind::Unknown $(&& !variant_has_kind!($v, kind))+,
            None => false,
        }
    };
}

/// The data match of `by_data!`: `$data` is the loaded data of a node of
/// file `$file_in`. The arms are those of `by_data!`.
macro_rules! match_data {
    ($file_in:expr, $data:expr, $def:expr, $([$($v:ident),+ $(,)?] => |$file:ident, $d:ident| $e:expr),+ $(,)?) => {{
        let file: usize = $file_in;
        match $data {
            $($(NodeData::$v($d) => {
                let $file = file;
                $e
            })+)+
            _ => $def,
        }
    }};
}

/// Go field read by node data variant. Each arm lists `NodeData` variants,
/// binds the variant data to `$d` and the node's file to `$file`, and
/// evaluates `$e`. Other variants give `$def`.
///
/// PERF: a published store node whose kind fits no listed variant gives `$def`
/// from its node record (`kind_lacks_data!`), without the pointer
/// chase to its astdata node and data.
macro_rules! by_data {
    ($n:expr, $def:expr, $([$($v:ident),+ $(,)?] => |$file:ident, $d:ident| $e:expr),+ $(,)?) => {{
        let node: Node = $n;
        if kind_lacks_data!(node, [$($($v),+),+]) {
            debug_assert!(!with_data!(node, |d| matches!(d, $($(NodeData::$v(_))|+)|+)));
            $def
        } else {
            with_data!(node, |d| {
                match_data!(node.file_index(), d, $def, $([$($v),+] => |$file, $d| $e),+)
            })
        }
    }};
}

/// Go list field read by node data variant: each arm lists `NodeData`
/// variants and the field (`req`, `opt` or `mods`, see `sel_field!`).
/// `$of` is `list_of` or `modifiers_of`. Other variants give `$def`. The
/// `kind_lacks_data!` shortcut is the same as in `by_data!`.
macro_rules! list_by_data {
    ($n:expr, $of:ident, $def:expr, $([$($v:ident),+ $(,)?] => $how:ident $field:ident),+ $(,)?) => {{
        let node: Node = $n;
        if kind_lacks_data!(node, [$($($v),+),+]) {
            $def
        } else {
            $of!(node, |d| match d {
                $($(NodeData::$v(d) => Some(sel_field!($how, d.$field)),)+)+
                _ => None,
            })
            .unwrap_or($def)
        }
    }};
}


/// Defines one Go field accessor as two `Node` methods from one arm list:
/// `$name(self)` reads the node like `by_data!`, and `$name_in(self, d)`
/// reads `d`, the data of the same node, which the caller already loaded
/// with `parsed_node_data`. `$n` names the node in the arms. The arms are
/// those of `by_data!`. With `d` `None` (a node that a freeable parse owns,
/// lsshells M3c, `LoadedData`) `$name_in` is `$name`.
///
/// PERF: query Q7-3. Each `$name` call repeats the `FROZEN` lookup, the
/// kind-table test and the data load. A caller that reads several fields of
/// one node (the binder) loads the data once and calls the `_in` methods.
/// Both methods give the same result.
macro_rules! data_accessor {
    (
        $(#[$attr:meta])*
        $vis:vis fn $name:ident, $name_in:ident($n:ident) -> $ty:ty {
            $def:expr,
            $([$($v:ident),+ $(,)?] => |$file:ident, $d:ident| $e:expr),+ $(,)?
        }
    ) => {
        $(#[$attr])*
        #[must_use]
        $vis fn $name(self) -> $ty {
            let $n: Node = self;
            by_data!($n, $def, $([$($v),+] => |$file, $d| $e),+)
        }

        #[doc = concat!("`Node::", stringify!($name), "` on `d`, the data of this node that the caller already loaded with `parsed_node_data` (see `data_accessor!`).")]
        #[must_use]
        $vis fn $name_in(self, d: LoadedData) -> $ty {
            let Some(d) = d else {
                return self.$name();
            };
            let $n: Node = self;
            debug_assert!(is_data_of(d, $n), "data of another node");
            match_data!($n.file_index(), d, $def, $([$($v),+] => |$file, $d| $e),+)
        }
    };
}

/// Expands `$mac! { $($args)* [variants] }` with the `NodeData` variants
/// that hold a Go `Modifiers()` list. `Node::modifiers` and the U1 (b) store
/// column (`store_node_modifier_bits`) both take the list from here, so they
/// agree by construction.
macro_rules! modifiers_variants {
    ($mac:ident! { $($args:tt)* }) => {
        $mac! {
            $($args)*
            [
                ArrowFunction,
                BinaryExpression,
                ClassDeclaration,
                ClassExpression,
                ClassStaticBlockDeclaration,
                ConstructorDeclaration,
                ConstructorTypeNode,
                EnumDeclaration,
                EnumMember,
                ExportAssignment,
                ExportDeclaration,
                FunctionDeclaration,
                FunctionExpression,
                FunctionTypeNode,
                GetAccessorDeclaration,
                SetAccessorDeclaration,
                ImportDeclaration,
                ImportEqualsDeclaration,
                IndexSignatureDeclaration,
                InterfaceDeclaration,
                MethodDeclaration,
                MethodSignatureDeclaration,
                MissingDeclaration,
                ModuleDeclaration,
                NamespaceExportDeclaration,
                ParameterDeclaration,
                PropertyAssignment,
                PropertyDeclaration,
                PropertySignatureDeclaration,
                ShorthandPropertyAssignment,
                TypeAliasDeclaration,
                TypeParameterDeclaration,
                VariableStatement
            ]
        }
    };
}

/// `Node::modifiers` and `Node::modifiers_in` over the `modifiers_variants!`
/// list, as `data_accessor!` does for other fields. The list is a handle, so
/// `modifiers` reads it with `modifiers_of!` (`list_by_data!`), not
/// `by_data!`.
macro_rules! modifiers_accessor {
    ([$($v:ident),+ $(,)?]) => {
        #[must_use]
        pub fn modifiers(self) -> ModifierList {
            list_by_data!(self, modifiers_of, ModifierList::NIL, [$($v),+] => mods modifiers)
        }

        /// `Node::modifiers` on `d`, the data of this node that the caller
        /// already loaded with `parsed_node_data` (see `data_accessor!`).
        #[must_use]
        pub fn modifiers_in(self, d: LoadedData) -> ModifierList {
            let Some(d) = d else {
                return self.modifiers();
            };
            debug_assert!(is_data_of(d, self), "data of another node");
            match_data!(
                self.file_index(),
                d,
                ModifierList::NIL,
                [$($v),+] => |f, d| mods(f, &d.modifiers)
            )
        }
    };
}

/// `store_node_modifier_bits` over the `modifiers_variants!` list.
macro_rules! store_node_modifier_bits_fn {
    ([$($v:ident),+ $(,)?]) => {
        /// U1 (b): `ModifierList::modifier_flags` of the own modifier list of
        /// `data`, the data of a store node of kind `kind`, as bits: the value
        /// `Node::modifiers().modifier_flags()` gives for it (store lists hold
        /// their Go flags). 0 when the node has no list. The kind test comes
        /// first, so most nodes are not loaded (the `NodeKids` modifier word).
        pub(crate) fn store_node_modifier_bits(kind: SyntaxKind, data: &NodeData) -> u32 {
            let mut has_list = false;
            $(has_list |= variant_has_kind!($v, kind);)+
            if !has_list {
                return 0;
            }
            match data {
                $(NodeData::$v(d) => d.modifiers.as_ref().map_or(0, |m| m.flags.0),)+
                _ => 0,
            }
        }
    };
}

modifiers_variants!(store_node_modifier_bits_fn! {});

// U4 (CH6, bind A): the plain field arms of Go `Name()`, `Expression()`,
// `PostfixToken()` and the own `QuestionToken` field. Each macro takes
// `$mac!($($pre)*)` and calls `$mac! { $($pre)* arms }`: it appends its arms
// to the tokens `$pre`, which end with a comma. The braces let the call
// expand to items (in `impl Node`) as well as to an expression. Each arm
// lists `NodeData` variants and reads the field with `$opt` (a field that
// can be nil) or `$req` (a required field). The accessors and the store
// column (`store_node_children`) both take the arms from here, so they
// agree by construction. The special arms (QualifiedName,
// CaseOrDefaultClause) stay with the accessors; the column marks those
// kinds unknown.

/// The plain arms of Go `Name()` (see above).
macro_rules! name_arms {
    ($mac:ident!($($pre:tt)*), $opt:ident, $req:ident) => {
        $mac! {
            $($pre)*
            [
                BindingElement,
                ClassDeclaration,
                ClassExpression,
                FunctionDeclaration,
                FunctionExpression,
                ImportClause,
                JsDocCallbackTag,
                JsDocLink,
                JsDocLinkCode,
                JsDocLinkPlain,
                JsDocTypedefTag,
            ] => |f, d| $opt(f, d.name),
            [
                EnumDeclaration,
                EnumMember,
                ExportSpecifier,
                GetAccessorDeclaration,
                SetAccessorDeclaration,
                ImportAttribute,
                ImportEqualsDeclaration,
                ImportSpecifier,
                InterfaceDeclaration,
                JsDocNameReference,
                JsDocParameterOrPropertyTag,
                JsxAttribute,
                JsxNamespacedName,
                MetaProperty,
                MethodDeclaration,
                MethodSignatureDeclaration,
                ModuleDeclaration,
                NamedTupleMember,
                NamespaceExport,
                NamespaceExportDeclaration,
                NamespaceImport,
                ParameterDeclaration,
                PropertyAccessExpression,
                PropertyAssignment,
                PropertyDeclaration,
                PropertySignatureDeclaration,
                ShorthandPropertyAssignment,
                TypeAliasDeclaration,
                TypeParameterDeclaration,
                VariableDeclaration,
            ] => |f, d| $req(f, d.name),
        }
    };
}

/// The plain arms of Go `Expression()` (see above).
// PORT: also covers TypeParameterDeclaration and SyntheticReferenceExpression,
// whose Go `Expression` fields have no generated accessor.
macro_rules! expression_arms {
    ($mac:ident!($($pre:tt)*), $opt:ident, $req:ident) => {
        $mac! {
            $($pre)*
            [
                PropertyAccessExpression,
                ElementAccessExpression,
                ParenthesizedExpression,
                CallExpression,
                NewExpression,
                ExpressionWithTypeArguments,
                ComputedPropertyName,
                NonNullExpression,
                TypeAssertion,
                AsExpression,
                SatisfiesExpression,
                TypeOfExpression,
                SpreadAssignment,
                SpreadElement,
                TemplateSpan,
                DeleteExpression,
                VoidExpression,
                AwaitExpression,
                PartiallyEmittedExpression,
                IfStatement,
                DoStatement,
                WhileStatement,
                WithStatement,
                ForInOrOfStatement,
                SwitchStatement,
                ExpressionStatement,
                ThrowStatement,
                ExternalModuleReference,
                ExportAssignment,
                Decorator,
                JsxSpreadAttribute,
                SyntheticReferenceExpression,
            ] => |f, d| $req(f, d.expression),
            [
                YieldExpression,
                ReturnStatement,
                JsxExpression,
                TypeParameterDeclaration,
            ] => |f, d| $opt(f, d.expression),
        }
    };
}

/// The arms of Go `PostfixToken()` (see above).
macro_rules! postfix_arms {
    ($mac:ident!($($pre:tt)*), $opt:ident) => {
        $mac! {
            $($pre)*
            [
                MethodDeclaration,
                ShorthandPropertyAssignment,
                MethodSignatureDeclaration,
                PropertySignatureDeclaration,
                PropertyAssignment,
                PropertyDeclaration,
                EnumMember,
                GetAccessorDeclaration,
                SetAccessorDeclaration,
            ] => |f, d| $opt(f, d.postfix_token),
        }
    };
}

/// The arms of the node's own Go `QuestionToken` field, the first step of
/// Go `QuestionToken()` (see above).
macro_rules! question_arms {
    ($mac:ident!($($pre:tt)*), $opt:ident, $req:ident) => {
        $mac! {
            $($pre)*
            [
                ParameterDeclaration,
                MappedTypeNode,
                NamedTupleMember,
            ] => |f, d| $opt(f, d.question_token),
            [ConditionalExpression] => |f, d| $req(f, d.question_token),
        }
    };
}

// C2: the arms of Go `Type()`, `Initializer()` and `TypeArgumentList()`,
// shared by the accessors and the `typed` field of the store column
// (`store_node_children`) in the same way as the U4 arms above. Every arm
// is a plain field read.

/// The arms of Go `Type()` (see above).
// PORT: also covers JsDocVariadicType, whose Go `Type` field has no
// generated accessor.
macro_rules! type_arms {
    ($mac:ident!($($pre:tt)*), $opt:ident, $req:ident) => {
        $mac! {
            $($pre)*
            [JsDocParameterOrPropertyTag] => |f, d| $opt(f, d.type_expression),
            [IndexSignatureDeclaration] => |f, d| $req(f, d.type_),
            [
                VariableDeclaration,
                ParameterDeclaration,
                PropertyDeclaration,
                PropertyAssignment,
                ShorthandPropertyAssignment,
                TypePredicateNode,
                MappedTypeNode,
                ExportAssignment,
                BinaryExpression,
                ArrowFunction,
                CallSignatureDeclaration,
                ConstructSignatureDeclaration,
                ConstructorDeclaration,
                FunctionDeclaration,
                FunctionExpression,
                GetAccessorDeclaration,
                SetAccessorDeclaration,
                JsDocSignature,
                MethodDeclaration,
                MethodSignatureDeclaration,
                FunctionTypeNode,
                ConstructorTypeNode,
            ] => |f, d| $opt(f, d.type_),
            [
                PropertySignatureDeclaration,
                ParenthesizedTypeNode,
                TypeOperatorNode,
                TypeAssertion,
                AsExpression,
                SatisfiesExpression,
                TypeAliasDeclaration,
                NamedTupleMember,
                OptionalTypeNode,
                RestTypeNode,
                TemplateLiteralTypeSpan,
                JsDocTypeExpression,
                JsDocNullableType,
                JsDocNonNullableType,
                JsDocOptionalType,
                JsDocVariadicType,
            ] => |f, d| $req(f, d.type_),
        }
    };
}

/// The arms of Go `Initializer()` (see above).
macro_rules! initializer_arms {
    ($mac:ident!($($pre:tt)*), $opt:ident, $req:ident) => {
        $mac! {
            $($pre)*
            [
                VariableDeclaration,
                ParameterDeclaration,
                BindingElement,
                PropertyDeclaration,
                EnumMember,
                ForStatement,
                JsxAttribute,
            ] => |f, d| $opt(f, d.initializer),
            [
                PropertySignatureDeclaration,
                PropertyAssignment,
                ForInOrOfStatement,
            ] => |f, d| $req(f, d.initializer),
        }
    };
}

/// The arms of Go `TypeArgumentList()` (see above). `$list` reads the
/// optional list field.
macro_rules! type_arguments_arms {
    ($mac:ident!($($pre:tt)*), $list:ident) => {
        $mac! {
            $($pre)*
            [
                CallExpression,
                NewExpression,
                TaggedTemplateExpression,
                TypeReferenceNode,
                ExpressionWithTypeArguments,
                ImportTypeNode,
                TypeQueryNode,
                JsxOpeningElement,
                JsxSelfClosingElement,
            ] => |f, d| $list(f, &d.type_arguments),
        }
    };
}

/// `Node::initializer_data` and `Node::initializer_data_in`: Go
/// `Initializer()` from the node data, over the `initializer_arms!` arms.
macro_rules! initializer_data_accessor {
    ($($arms:tt)*) => {
        data_accessor! {
            fn initializer_data, initializer_data_in(n) -> Node {
                Node::NIL,
                $($arms)*
            }
        }
    };
}

/// One `match_data!` over the expression, postfix and own question arms for
/// the `other` entry of `store_node_children`: `Some((tag, id))`, or `None`
/// for unknown. The three field sets have disjoint variants, so one match
/// covers them (a variant in two sets would be an unreachable pattern). The
/// `@` steps collect the arms of each list macro in turn.
macro_rules! other_column_match {
    (@expression $data:expr, $expr_opt:ident, $expr_req:ident, $postfix:ident, $q_opt:ident, $q_req:ident) => {
        expression_arms!(
            other_column_match!(@postfix $data, $postfix, $q_opt, $q_req,),
            $expr_opt,
            $expr_req
        )
    };
    (@postfix $data:expr, $postfix:ident, $q_opt:ident, $q_req:ident, $($arms:tt)*) => {
        postfix_arms!(other_column_match!(@question $data, $q_opt, $q_req, $($arms)*), $postfix)
    };
    (@question $data:expr, $q_opt:ident, $q_req:ident, $($arms:tt)*) => {
        question_arms!(other_column_match!(@match $data, $($arms)*), $q_opt, $q_req)
    };
    (@match $data:expr, $($arms:tt)*) => {
        match_data!(
            0,
            $data,
            // A kind with none of the fields: every read gives nil.
            Some((SlotChildren::TAG_EXPRESSION, 0)),
            // The special arms of `Node::expression`.
            [CaseOrDefaultClause, QualifiedName] => |_f, _d| None,
            $($arms)*
        )
    };
}

/// `Node::name_data` and `Node::name_data_in`: Go `Name()` from the node
/// data, over the `name_arms!` arms.
macro_rules! name_data_accessor {
    ($($arms:tt)*) => {
        data_accessor! {
            fn name_data, name_data_in(n) -> Node {
                Node::NIL,
                // PORT: a QualifiedName that Go parses as a PropertyAccessExpression.
                [QualifiedName] => |f, d| if n.kind() == SyntaxKind::PropertyAccessExpression {
                    req(f, d.right)
                } else {
                    Node::NIL
                },
                $($arms)*
            }
        }
    };
}

/// `Node::postfix_token_data` and `Node::postfix_token_data_in`: Go
/// `PostfixToken()` from the node data, over the `postfix_arms!` arms.
macro_rules! postfix_token_data_accessor {
    ($($arms:tt)*) => {
        data_accessor! {
            fn postfix_token_data, postfix_token_data_in(n) -> Node {
                Node::NIL,
                $($arms)*
            }
        }
    };
}

/// `Node::own_question_token` and `Node::own_question_token_in` over the
/// `question_arms!` arms: `Some` of the node's own question token field,
/// `None` for kinds without that field (the first step of Go
/// `QuestionToken`).
macro_rules! own_question_token_accessor {
    ($($arms:tt)*) => {
        data_accessor! {
            fn own_question_token, own_question_token_in(n) -> Option<Node> {
                None,
                $($arms)*
            }
        }
    };
}



/// The `list_of!` selector result of an optional type argument list field,
/// for `Node::type_argument_list` over the `type_arguments_arms!` arms.
fn sel_type_arguments(
    _file: usize,
    l: &Option<crate::astdata::NodeList>,
) -> Option<Option<AnyList<'_>>> {
    Some(l.as_ref().map(AnyList::Nodes))
}

/// A `SlotChildren` id of an optional child field: 0 (the nil slot) for Go
/// nil.
fn column_opt_id(id: Option<crate::astdata::NodeId>) -> u32 {
    id.map_or(0, column_req_id)
}

/// A `SlotChildren` id of a required child field.
fn column_req_id(id: crate::astdata::NodeId) -> u32 {
    // Child ids are `u32` slot indexes.
    id.index() as u32
}

/// U4 (CH6, bind A): the `SlotChildren` entry of a store node of kind `kind`
/// with astdata node `node`, from the arm lists of `Node::name`,
/// `Node::expression`, `Node::postfix_token` and `Node::question_token`,
/// and (C2) of `Node::type_`, `Node::initializer`, `Node::type_name` and
/// `Node::type_argument_list`. Store data always fits the header kind
/// (`alloc_store_node`). A kind with a special arm (QualifiedName,
/// CaseOrDefaultClause) gets an unknown field, whose reads take the node
/// data.
// PERF: called when the slot is made, while its data is hot (a few data
// matches), so no pass over the node data is needed later.
pub(crate) fn store_node_children(kind: SyntaxKind, data: &NodeData) -> SlotChildren {
    debug_assert!(
        data.matches_syntax_kind(kind),
        "{kind:?} does not fit its NodeData"
    );
    // PERF: about a third of the slots are identifiers, which have none of
    // the fields. The early exit skips the data matches.
    if kind == SyntaxKind::Identifier {
        let none = SlotChildren::new(Some(0), Some((SlotChildren::TAG_EXPRESSION, 0)))
            .with_typed(Some((SlotChildren::TYPED_TYPE, 0)), true);
        debug_assert_eq!(none, store_node_children_from_data(data));
        return none;
    }
    store_node_children_from_data(data)
}

/// `store_node_children` for node data `data`.
fn store_node_children_from_data(data: &NodeData) -> SlotChildren {
    let id_opt = |_: usize, id: Option<crate::astdata::NodeId>| Some(column_opt_id(id));
    let id_req = |_: usize, id: crate::astdata::NodeId| Some(column_req_id(id));
    let name = name_arms!(
        match_data!(0, data, Some(0), [QualifiedName] => |_f, _d| None,),
        id_opt,
        id_req
    );
    let tagged_opt = |tag: u32| {
        move |_: usize, id: Option<crate::astdata::NodeId>| Some((tag, column_opt_id(id)))
    };
    let tagged_req =
        |tag: u32| move |_: usize, id: crate::astdata::NodeId| Some((tag, column_req_id(id)));
    let (expr_opt, expr_req) = (
        tagged_opt(SlotChildren::TAG_EXPRESSION),
        tagged_req(SlotChildren::TAG_EXPRESSION),
    );
    let postfix = tagged_opt(SlotChildren::TAG_POSTFIX);
    let (q_opt, q_req) = (
        tagged_opt(SlotChildren::TAG_QUESTION),
        tagged_req(SlotChildren::TAG_QUESTION),
    );
    let other = other_column_match!(@expression data, expr_opt, expr_req, postfix, q_opt, q_req);
    // C2: `None` for a kind without the field, `Some(0)` for nil. A kind can
    // have both a type and an initializer, so each field has its own match.
    // The column holds the one that is set; only a node where both are set
    // keeps its initializer in the data (`TYPED_TYPE_WITH_INITIALIZER`).
    let type_id = type_arms!(match_data!(0, data, None,), id_opt, id_req);
    let initializer_id = initializer_arms!(match_data!(0, data, None,), id_opt, id_req);
    let typed = match (type_id, initializer_id) {
        (Some(ty), Some(init)) if ty != 0 && init != 0 => {
            (SlotChildren::TYPED_TYPE_WITH_INITIALIZER, ty)
        }
        (Some(ty), _) if ty != 0 => (SlotChildren::TYPED_TYPE, ty),
        (_, Some(init)) => (SlotChildren::TYPED_INITIALIZER, init),
        (Some(_), None) => (SlotChildren::TYPED_TYPE, 0),
        (None, None) => match data {
            // Go `AsTypeReference().TypeName` (`Node::type_name`).
            NodeData::TypeReferenceNode(d) => {
                (SlotChildren::TYPED_TYPE_NAME, column_req_id(d.type_name))
            }
            _ => (SlotChildren::TYPED_TYPE, 0),
        },
    };
    // Only a missing list gives `NodeList::NIL` itself.
    let list_is_none = |_: usize, list: &Option<crate::astdata::NodeList>| list.is_none();
    let no_type_arguments = type_arguments_arms!(match_data!(0, data, true,), list_is_none);
    SlotChildren::new(name, other).with_typed(Some(typed), no_type_arguments)
}

/// Expands `$mac! { $($args)* [variants] }` with the `NodeData` variants of
/// the Go node types that embed `LocalsContainerBase`. `is_locals_container`
/// and the U1 (d) kind test of `Node::locals` both take the list from here,
/// so they agree by construction.
macro_rules! locals_container_variants {
    ($mac:ident! { $($args:tt)* }) => {
        $mac! {
            $($args)*
            [
                ArrowFunction,
                Block,
                CallSignatureDeclaration,
                CaseBlock,
                CatchClause,
                ClassDeclaration,
                ClassExpression,
                ClassStaticBlockDeclaration,
                ConditionalTypeNode,
                ConstructSignatureDeclaration,
                ConstructorDeclaration,
                ConstructorTypeNode,
                ForInOrOfStatement,
                ForStatement,
                FunctionDeclaration,
                FunctionExpression,
                FunctionTypeNode,
                GetAccessorDeclaration,
                IndexSignatureDeclaration,
                JsDocSignature,
                MappedTypeNode,
                MethodDeclaration,
                MethodSignatureDeclaration,
                ModuleDeclaration,
                SetAccessorDeclaration,
                SourceFile,
                TypeAliasDeclaration
            ]
        }
    };
}

/// True when `$d` (a `&NodeData`) is one of the listed variants.
macro_rules! data_is_variant {
    ($d:expr, [$($v:ident),+ $(,)?]) => {
        matches!($d, $(NodeData::$v(_))|+)
    };
}



/// Binder data for nodes of a file that is not bound yet.
static NO_BIND: NodeBindData = NodeBindData {
    symbol: SymbolId::NIL,
    local_symbol: SymbolId::NIL,
    locals: SymbolTable::NIL,
    next_container: Node::NIL,
    flow_node: FlowNodeId::NIL,
    end_flow_node: FlowNodeId::NIL,
    return_flow_node: FlowNodeId::NIL,
    added_flags: NodeFlags::NONE,
};


// ──────────────────────────────────────────────────────────────────────
// core.TextRange (internal/core/text.go)
// ──────────────────────────────────────────────────────────────────────

/// Go `core.TextRange`. Positions are Go positions (UTF-8 byte offsets).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TextRange {
    pos: i32,
    end: i32,
}

impl TextRange {
    // Go: core/text.go:14 NewTextRange
    #[must_use]
    pub const fn new(pos: i32, end: i32) -> Self {
        Self { pos, end }
    }

    // Go: core/text.go:18 UndefinedTextRange
    #[must_use]
    pub const fn undefined() -> Self {
        Self { pos: -1, end: -1 }
    }

    // Go: core/text.go:22 Pos
    #[must_use]
    pub const fn pos(self) -> i32 {
        self.pos
    }

    // Go: core/text.go:26 End
    #[must_use]
    pub const fn end(self) -> i32 {
        self.end
    }

    // Go: core/text.go:30 Len
    #[must_use]
    pub const fn len(self) -> i32 {
        self.end - self.pos
    }

    // Go: core/text.go:34 IsValid
    #[must_use]
    pub const fn is_valid(self) -> bool {
        self.pos >= 0 || self.end >= 0
    }

    // Go: core/text.go:38 Contains
    #[must_use]
    pub const fn contains(self, pos: i32) -> bool {
        pos >= self.pos && pos < self.end
    }

    // Go: core/text.go:42 ContainsInclusive
    #[must_use]
    pub const fn contains_inclusive(self, pos: i32) -> bool {
        pos >= self.pos && pos <= self.end
    }

    // Go: core/text.go:46 ContainsExclusive
    #[must_use]
    pub const fn contains_exclusive(self, pos: i32) -> bool {
        self.pos < pos && pos < self.end
    }

    // Go: core/text.go:50 WithPos
    #[must_use]
    pub const fn with_pos(self, pos: i32) -> Self {
        Self { pos, end: self.end }
    }

    // Go: core/text.go:54 WithEnd
    #[must_use]
    pub const fn with_end(self, end: i32) -> Self {
        Self { pos: self.pos, end }
    }

    // Go: core/text.go:58 ContainedBy
    #[must_use]
    pub const fn contained_by(self, t2: TextRange) -> bool {
        t2.pos <= self.pos && t2.end >= self.end
    }

    // Go: core/text.go:62 Overlaps
    #[must_use]
    pub fn overlaps(self, t2: TextRange) -> bool {
        let start = self.pos.max(t2.pos);
        let end = self.end.min(t2.end);
        start < end
    }

    // Go: core/text.go:70 Intersects
    /// Like `overlaps`, but touching ranges intersect.
    #[must_use]
    pub fn intersects(self, t2: TextRange) -> bool {
        let start = self.pos.max(t2.pos);
        let end = self.end.min(t2.end);
        start <= end
    }
}

// Go: core/text.go:76 CompareTextRanges
#[must_use]
pub fn compare_text_ranges(r1: TextRange, r2: TextRange) -> i32 {
    let c = r1.pos - r2.pos;
    if c != 0 {
        return c;
    }
    r1.end - r2.end
}

/// A astdata range as a Go `core.TextRange`.
fn text_range_of(r: &crate::astdata::text::TextRange) -> TextRange {
    TextRange::new(r.start.get() as i32, r.end.get() as i32)
}




// ──────────────────────────────────────────────────────────────────────
// NodeSlice: Go `[]*Node`
// ──────────────────────────────────────────────────────────────────────

/// Go `[]*Node`. It points at a astdata id list of one file, at a slice of
/// `Node`s that lives for the process, or at a list of this thread's
/// synthetic arena. The default value is Go `nil`.
#[derive(Clone, Copy, Debug)]
pub struct NodeSlice(SliceRepr);

#[derive(Clone, Copy, Debug)]
enum SliceRepr {
    /// Ts_ast ids in `file`.
    Ids {
        file: u32,
        ids: &'static [crate::astdata::NodeId],
    },
    /// Nodes that live for the process.
    Nodes(&'static [Node]),
    /// The `len` nodes of a list of this thread's synthetic arena.
    Synthetic { list: SyntheticList, len: u32 },
    /// A list of `len` nodes of freeable file version `file` (lsshells M3b),
    /// read at each use (`file_list_node`): Go `SourceFile.Imports()` when
    /// `host` is nil (`NodeSlice::from_file_imports`), else the eager JSDoc
    /// of `host` in its JSDoc cache (`NodeSlice::from_file_js_doc`; before
    /// the publish, the cache of its store, lsshells M3c). A published list
    /// does not change, so its length is kept here.
    FileList { file: u32, len: u32, host: Node },
    /// The `len` nodes of a list of a store that owns its nodes (a freeable
    /// parse, lsshells M3c), read at each use (`with_store_list`).
    Store { list: StoreList, len: u32 },
}


impl NodeSlice {
    /// Go `nil`.
    pub const NIL: Self = Self(SliceRepr::Nodes(&[]));

    /// A slice over astdata ids in `file`.
    #[must_use]
    pub fn from_ids(file: usize, ids: &'static [crate::astdata::NodeId]) -> Self {
        Self(SliceRepr::Ids {
            file: file as u32,
            ids,
        })
    }

    /// A slice over nodes that live for the program.
    #[must_use]
    pub fn from_nodes(nodes: &'static [Node]) -> Self {
        Self(SliceRepr::Nodes(nodes))
    }






    #[inline]
    #[must_use]
    pub fn len(self) -> usize {
        match self.0 {
            SliceRepr::Ids { ids, .. } => ids.len(),
            SliceRepr::Nodes(nodes) => nodes.len(),
            SliceRepr::Synthetic { len, .. } => len as usize,
            SliceRepr::FileList { len, .. } => len as usize,
            SliceRepr::Store { len, .. } => len as usize,
        }
    }

    #[must_use]
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }





}




/// The most nodes of a store list that `NodeSliceIter` reads once
/// (`SliceIds::Inline`).
const INLINE_NODES: usize = 4;

/// Entries of `HOT_LIST_NODES`.
const HOT_LIST_NODE_SLOTS: usize = 256;





// PERF: 24 bytes. The synthetic list is a variant of `SyntheticList`, whose
// tag and index fit next to the length.
const _: () = assert!(std::mem::size_of::<NodeSlice>() == 24);

/// How a `NodeSliceIter` turns position `i` into a node, read once for the
/// whole loop.
#[derive(Clone, Copy, Debug)]
enum SliceIds {
    /// The nodes of the slice (`NodeSlice::from_nodes`).
    Nodes(&'static [Node]),
    /// The ids of the slice, of a frozen store (`frozen_store_ids`).
    Frozen(&'static [crate::astdata::NodeId], FrozenIds),
    /// A list of a store that owns its nodes, read at each step
    /// (`store_list_node`, lsshells M3f).
    Store(StoreList),
    /// The nodes of a short list of a store that owns its nodes, read once
    /// (`store_list_first_nodes`, lsshells M3f).
    Inline([Node; INLINE_NODES]),
    /// `NodeSlice::get` per node: ids before freeze, synthetic lists and
    /// the file lists of freeable file versions.
    Slow,
}

/// Iterator over a `NodeSlice`. Yields `Node` by value.
#[derive(Clone, Debug)]
pub struct NodeSliceIter {
    slice: NodeSlice,
    ids: SliceIds,
    front: usize,
    back: usize,
}

impl NodeSliceIter {
}






// ──────────────────────────────────────────────────────────────────────
// NodeList
// ──────────────────────────────────────────────────────────────────────








// ──────────────────────────────────────────────────────────────────────
// ModifierList
// ──────────────────────────────────────────────────────────────────────








// ──────────────────────────────────────────────────────────────────────
// Visitor helpers
// ──────────────────────────────────────────────────────────────────────

// Go: ast.go:31 visit
fn visit(v: &mut dyn FnMut(Node) -> bool, node: Node) -> bool {
    if node.is_some() {
        return v(node);
    }
    false
}

// Go: ast.go:38 visitNodes

// Go: ast.go:47 visitNodeList

// Go: ast.go:54 visitModifiers

// ──────────────────────────────────────────────────────────────────────
// Node core accessors
// ──────────────────────────────────────────────────────────────────────


/// The node indexes that a facts column holds (`SUBTREE_FACTS`). A larger
/// index (a file of over 4 million nodes) goes in `SUBTREE_FACTS_SPARSE`,
/// so a column never holds more than 16 MiB.
const FACTS_COLUMN_LIMIT: usize = 1 << 22;

/// The index of non-nil node `n` in its file (`SUBTREE_FACTS`).
#[inline]
fn facts_index(n: Node) -> usize {
    (n.0 & 0xffff_ffff).wrapping_sub(1) as usize
}

/// The `SUBTREE_FACTS` key of the file of `n`: the handle of index 0 of
/// that file, so `PerFileMap` forgets it with the file version.
#[inline]
fn facts_file_key(n: Node) -> Node {
    Node((n.0 & !0xffff_ffff) | 1)
}

/// The cached `Node::subtree_facts` of `n`, a node of a `CompositeBase`
/// kind, if any. A node of a static publish keeps them in the facts column
/// of its file (`static_facts_word`, factscol1b), a factory node in its
/// arena slot (emitast1 F1), any other node in `SUBTREE_FACTS`.
#[cfg(test)]
fn cached_subtree_facts(n: Node) -> Option<SubtreeFacts> {
    if let Some(word) = super::store::static_facts_word(n) {
        return computed_facts(SubtreeFacts(word.load(Ordering::Relaxed)));
    }
    cached_unpublished_subtree_facts(n)
}




impl Node {









    // Go: ast.go:192 Pos

    // Go: ast.go:193 End


    // Go: ast.go:198 Name
    // PERF: U4 (CH6). A published store node reads its name child from the
    // store column (`frozen_store_child`), not from its node data. Debug
    // builds compare it with the data read.


    // `name_data` and `name_data_in`: Go `Name()` from the node data.

    // Go: ast.go:199 Modifiers

    // Go: ast.go:217 ParameterList

    // Go: ast.go:218 Parameters

    // Go: ast.go:209 SubtreeFacts

    // Go: ast.go:229 Decorators

    // Go: ast.go:241 Symbol
    // PORT: Go reads `DeclarationData().Symbol`. The binder only sets the
    // symbol on declaration nodes, so reading the bind data directly is the
    // same.
    // PERF: AST node records, step 2. The symbol of a published store node
    // is in its record (`frozen_store_symbol`).

    // Go: ast.go:249 LocalSymbol

    // Go: ast.go:257 Locals
    // PERF: U1 (d). Go `Locals()` is nil when `LocalsContainerData()` is nil.
    // A frozen store node whose kind is no locals container gives nil from
    // the kind table, without the binder data lookup. The binder sets locals
    // only on locals containers (debug_asserts in binder_p1 and binder_p2).







}


impl FlowNodeId {




}







// ──────────────────────────────────────────────────────────────────────
// Node field accessors (Go methods on *Node)
// ──────────────────────────────────────────────────────────────────────



impl Node {
    // Go: ast.go:265 Body
    // PORT: also covers ClassStaticBlockDeclaration, whose Go `Body` field
    // has no generated accessor.

    // Go: ast.go:261 Text
    // PORT: also covers JsxText, whose Go `Text` field has no generated
    // accessor. Other kinds give "" instead of a panic.





    // Go: ast.go:311 Expression
    // PERF: U4 (CH6). A published store node reads its expression child from
    // the store column (`frozen_store_child`), as `Node::name` does.


    // Go: ast.go:387 RawText
    // PORT: also covers NoSubstitutionTemplateLiteral, whose Go `RawText`
    // field has no generated accessor.

    // Go: ast.go:477 ArgumentList

    // Go: ast.go:487 Arguments

    // Go: ast.go:495 TypeArgumentList
    // PERF: C2. A published store node whose data has no list gives nil from
    // the store column (`frozen_store_lacks_type_arguments`), without its
    // node data. A node with a list reads it from the data.


    // Go: ast.go:519 TypeArguments

    // Go: ast.go:527 TypeParameterList

    // Go: ast.go:548 TypeParameters

    // Go: ast.go:556 MemberList

    // Go: ast.go:574 Members

    // Go: ast.go:582 StatementList

    // Go: ast.go:596 Statements

    // Go: ast.go:604 CanHaveStatements

    // Go: ast.go:613 ModifierFlags
    // PERF: U1 (b). A frozen store node reads its flags from the store column
    // (`frozen_store_modifier_flags`), without its node data and list.

    // Go: ast.go:621 ModifierNodes

    // Go: ast.go:629 Type
    // PORT: also covers JsDocVariadicType, whose Go `Type` field has no
    // generated accessor.
    // PERF: C2. A published store node reads its type child from the store
    // column (`frozen_store_child`), as `Node::name` does.


    // Go: ast.go:751 Initializer
    // PERF: C2, as `Node::type_`. A node with both a type and an
    // initializer keeps only the type in the column; its initializer reads
    // take the node data.


    // `initializer_data` and `initializer_data_in`: Go `Initializer()` from
    // the node data.

    // Go: ast.go:805 TagName

    // Go: ast.go:859 PropertyName

    // Go: ast.go:871 PropertyNameOrName

    // Go: ast.go:879 IsTypeOnly

    // Go: ast.go:896 CommentList

    // Go: ast.go:946 Comments

    // Go: ast.go:954 Label

    // Go: ast.go:966 Attributes
    // PORT: also covers the `Attributes` fields of ImportDeclaration,
    // ExportDeclaration, ImportTypeNode and JSDocImportTag, which have no
    // generated accessor.

    // Go: ast.go:978 Children

    // Go: ast.go:988 ModuleSpecifier

    // Go: ast.go:1000 ImportClause

    // Go: ast.go:1010 Statement

    // Go: ast.go:1028 PropertyList

    // Go: ast_generated.go ImportAttributes.Attributes

    // Go: ast.go:1038 Properties

    // Go: ast.go:1046 ElementList

    // Go: ast.go:1062 Elements

    // Go: ast.go:1070 PostfixToken
    // PERF: U4 (bind A). A published store node reads the token from the store
    // column (`frozen_store_child`), as `Node::name` does.


    // `postfix_token_data` and `postfix_token_data_in`: Go `PostfixToken()`
    // from the node data.

    // Go: ast.go:1094 QuestionToken
    // PERF: U4 (bind A), as `Node::postfix_token`.




    // `own_question_token` and `own_question_token_in`: the first step of Go
    // `QuestionToken`.

    // Go: ast.go:1112 QuestionDotToken

    // Go: ast.go:1126 TypeExpression
    // PORT: also covers the JSDoc `@this` and `@overload` tags, whose Go
    // `TypeExpression` fields have no generated accessor.

    // Go: ast.go:1146 ClassName

    // Go: ast.go:1158 Contains
}

// ──────────────────────────────────────────────────────────────────────
// ForEachChild / IterChildren
// ──────────────────────────────────────────────────────────────────────

impl Node {
    // Go: ast.go:194 ForEachChild

    // Go: ast.go:196 IterChildren
}

// Go: ast_generated.go ForEachChild (one method per node struct)
// PORT: the per-struct Go methods are merged into one match. The order of
// each arm follows the generated Go code.


impl Node {
}



/// R3-2: calls `v` with the slot index of each child id in `data`, the
/// data of a store node of Go kind `kind`, in Go `ForEachChild` order, and
/// stops when `v` returns true. It reads only the node data: no store and
/// no node read. A nil single child field is skipped, as Go `visit` skips
/// nil; list entries are all passed, so `v` sees slot 0 (nil) for a nil
/// list entry. `v` must resolve each index in the store of `node`: a node
/// slot is that node, and any other slot (nil, alias) is not.
// PERF: R3-2. The parser sets the parents of the children of each node it
// finishes (`set_parent_in_store_children`). The generic walk read the
// data and every child through the node reads (`static_ast_node_slow`,
// `Node::new_slow`), a store lookup each.
pub(crate) fn for_each_store_child_id(
    kind: SyntaxKind,
    data: &NodeData,
    v: impl FnMut(u32) -> bool,
) -> bool {
    walk_children(data, &mut StoreChildIds { kind, v })
}

/// R3-2: the fields that one arm of `walk_children` visits, in Go
/// `ForEachChild` order. Each method returns true to stop the walk. The
/// generic walk (`NodeChildVisit`) turns the ids into `Node` handles; the
/// parser's walk over a store node (`StoreChildIds`) keeps the slot
/// indexes. One match serves both, so they visit the same fields. `'d` is
/// the lifetime of the node data: `'static` for a parsed node of a static
/// parse, a read scope for a factory node and for a node that a freeable
/// parse owns (`ScopedChildVisit`, `StoreChildIds`).
trait ChildVisit<'d> {
    /// A required child field (Go `visit`).
    fn node(&mut self, id: crate::astdata::NodeId) -> bool;
    /// An optional child field (Go `visit`).
    fn opt(&mut self, id: Option<crate::astdata::NodeId>) -> bool;
    /// A required list field (Go `visitNodeList`).
    fn list(&mut self, l: &'d crate::astdata::NodeList) -> bool;
    /// An optional list field (Go `visitNodeList`).
    fn opt_list(&mut self, l: &'d Option<crate::astdata::NodeList>) -> bool;
    /// A modifier list field (Go `visitModifiers`).
    fn mods(&mut self, m: &'d Option<crate::astdata::ModifierList>) -> bool;
    /// A Go `[]*Node` field with no list (Go `visitNodes`).
    fn ids(&mut self, ids: &'d [crate::astdata::NodeId]) -> bool;
    /// Go `CaseOrDefaultClause.Expression` (see `case_expression`).
    fn case_expression(&mut self, id: crate::astdata::NodeId) -> bool;
    /// The Go `FullSignature` field of a function-like node: an optional
    /// child field (Go `visit`). Only `ChildFields` tells it apart.
    #[inline(always)]
    fn full_signature(&mut self, id: Option<crate::astdata::NodeId>) -> bool {
        self.opt(id)
    }
}







/// One child field of node data, in Go `ForEachChild` order
/// (`for_each_child_field`).
pub(crate) enum ChildField<'d> {
    /// A single child (Go `visit`). `Node::NIL` for Go `nil`.
    Node(Node),
    /// The `FullSignature` child of a function-like node (Go `visit`).
    /// `Node::NIL` for Go `nil`. The API encoder's property mask has no bit
    /// for it.
    FullSignature(Node),
    /// A list (Go `visitNodeList`). `None` for Go `nil`: no list, or the nil
    /// marker of a required list field (`is_nil_list_marker`).
    List(Option<&'d crate::astdata::NodeList>),
    /// A modifier list (Go `visitModifiers`). `None` for Go `nil`.
    Modifiers(Option<&'d crate::astdata::ModifierList>),
    /// A Go `[]*Node` field with no list (Go `visitNodes`): the children of
    /// a SyntaxList and the property tags of a JSDocTypeLiteral.
    Ids(&'d [crate::astdata::NodeId]),
}


/// The walk of `for_each_child_field`. It never stops early.
struct ChildFields<F> {
    /// The Go kind of the node (`case_expression`).
    kind: SyntaxKind,
    file: usize,
    f: F,
}


/// R3-2: the walk of `for_each_store_child_id`. The ids are slot indexes of
/// the store of the node. Slot 0 resolves to Go `nil`
/// (`resolve_store_id`), so a single child field with id 0 is skipped.
struct StoreChildIds<F> {
    /// The Go kind of the node (`case_expression` reads it).
    kind: SyntaxKind,
    v: F,
}

impl<'d, F: FnMut(u32) -> bool> ChildVisit<'d> for StoreChildIds<F> {
    #[inline(always)]
    fn node(&mut self, id: crate::astdata::NodeId) -> bool {
        id.index() != 0 && (self.v)(id.index() as u32)
    }

    #[inline(always)]
    fn opt(&mut self, id: Option<crate::astdata::NodeId>) -> bool {
        id.is_some_and(|id| self.node(id))
    }

    // `NodeList::is_nil` of a store list: the nil marker list.
    #[inline(always)]
    fn list(&mut self, l: &'d crate::astdata::NodeList) -> bool {
        !is_nil_list_marker(l) && self.ids(&l.nodes)
    }

    #[inline(always)]
    fn opt_list(&mut self, l: &'d Option<crate::astdata::NodeList>) -> bool {
        l.as_ref().is_some_and(|l| self.list(l))
    }

    // `ModifierList::is_nil` is only `None`.
    #[inline(always)]
    fn mods(&mut self, m: &'d Option<crate::astdata::ModifierList>) -> bool {
        m.as_ref().is_some_and(|m| self.ids(&m.list.nodes))
    }

    #[inline(always)]
    fn ids(&mut self, ids: &'d [crate::astdata::NodeId]) -> bool {
        ids.iter().any(|id| (self.v)(id.index() as u32))
    }

    #[inline(always)]
    fn case_expression(&mut self, id: crate::astdata::NodeId) -> bool {
        self.kind == SyntaxKind::CaseClause && self.node(id)
    }
}

/// The Go `ForEachChild` fields of node data `data`, in Go order, for `w`.
// Go: ast_generated.go ForEachChild (one method per node struct)
// PORT: the per-struct Go methods are merged into one match. The order of
// each arm follows the generated Go code.
fn walk_children<'d>(data: &'d NodeData, w: &mut impl ChildVisit<'d>) -> bool {
    macro_rules! n {
        ($x:expr) => {
            w.node($x)
        };
    }
    macro_rules! o {
        ($x:expr) => {
            w.opt($x)
        };
    }
    macro_rules! l {
        ($x:expr) => {
            w.list(&$x)
        };
    }
    macro_rules! ol {
        ($x:expr) => {
            w.opt_list(&$x)
        };
    }
    macro_rules! m {
        ($x:expr) => {
            w.mods(&$x)
        };
    }
    match data {
        NodeData::QualifiedName(d) => n!(d.left) || n!(d.right),
        NodeData::ComputedPropertyName(d) => n!(d.expression),
        NodeData::Decorator(d) => n!(d.expression),
        NodeData::IfStatement(d) => {
            n!(d.expression) || n!(d.then_statement) || o!(d.else_statement)
        }
        NodeData::DoStatement(d) => n!(d.statement) || n!(d.expression),
        NodeData::WhileStatement(d) => n!(d.expression) || n!(d.statement),
        NodeData::ForStatement(d) => {
            o!(d.initializer) || o!(d.condition) || o!(d.incrementor) || n!(d.statement)
        }
        NodeData::ForInOrOfStatement(d) => {
            o!(d.await_modifier) || n!(d.initializer) || n!(d.expression) || n!(d.statement)
        }
        NodeData::BreakStatement(d) => o!(d.label),
        NodeData::ContinueStatement(d) => o!(d.label),
        NodeData::ReturnStatement(d) => o!(d.expression),
        NodeData::WithStatement(d) => n!(d.expression) || n!(d.statement),
        NodeData::SwitchStatement(d) => n!(d.expression) || n!(d.case_block),
        NodeData::CaseBlock(d) => l!(d.clauses),
        NodeData::CaseOrDefaultClause(d) => w.case_expression(d.expression) || l!(d.statements),
        NodeData::ThrowStatement(d) => n!(d.expression),
        NodeData::TryStatement(d) => n!(d.try_block) || o!(d.catch_clause) || o!(d.finally_block),
        NodeData::CatchClause(d) => o!(d.variable_declaration) || n!(d.block),
        NodeData::LabeledStatement(d) => n!(d.label) || n!(d.statement),
        NodeData::ExpressionStatement(d) => n!(d.expression),
        NodeData::Block(d) => l!(d.statements),
        NodeData::VariableStatement(d) => m!(d.modifiers) || n!(d.declaration_list),
        NodeData::VariableDeclaration(d) => {
            n!(d.name) || o!(d.exclamation_token) || o!(d.type_) || o!(d.initializer)
        }
        NodeData::VariableDeclarationList(d) => l!(d.declarations),
        NodeData::BindingPattern(d) => l!(d.elements),
        NodeData::ParameterDeclaration(d) => {
            m!(d.modifiers)
                || o!(d.dot_dot_dot_token)
                || n!(d.name)
                || o!(d.question_token)
                || o!(d.type_)
                || o!(d.initializer)
        }
        NodeData::BindingElement(d) => {
            o!(d.dot_dot_dot_token) || o!(d.property_name) || o!(d.name) || o!(d.initializer)
        }
        NodeData::MissingDeclaration(d) => m!(d.modifiers),
        NodeData::FunctionDeclaration(d) => {
            m!(d.modifiers)
                || o!(d.asterisk_token)
                || o!(d.name)
                || ol!(d.type_parameters)
                || l!(d.parameters)
                || o!(d.type_)
                || w.full_signature(d.full_signature)
                || o!(d.body)
        }
        NodeData::ClassDeclaration(d) => {
            m!(d.modifiers)
                || o!(d.name)
                || ol!(d.type_parameters)
                || ol!(d.heritage_clauses)
                || l!(d.members)
        }
        NodeData::ClassExpression(d) => {
            m!(d.modifiers)
                || o!(d.name)
                || ol!(d.type_parameters)
                || ol!(d.heritage_clauses)
                || l!(d.members)
        }
        NodeData::InterfaceDeclaration(d) => {
            m!(d.modifiers)
                || n!(d.name)
                || ol!(d.type_parameters)
                || ol!(d.heritage_clauses)
                || l!(d.members)
        }
        NodeData::HeritageClause(d) => l!(d.types),
        NodeData::TypeAliasDeclaration(d) => {
            m!(d.modifiers) || n!(d.name) || ol!(d.type_parameters) || n!(d.type_)
        }
        NodeData::EnumMember(d) => n!(d.name) || o!(d.initializer),
        NodeData::EnumDeclaration(d) => m!(d.modifiers) || n!(d.name) || l!(d.members),
        NodeData::ModuleBlock(d) => l!(d.statements),
        NodeData::ImportDeclaration(d) => {
            m!(d.modifiers) || o!(d.import_clause) || n!(d.module_specifier) || o!(d.attributes)
        }
        NodeData::ExternalModuleReference(d) => n!(d.expression),
        NodeData::NamespaceImport(d) => n!(d.name),
        NodeData::NamedImports(d) => l!(d.elements),
        NodeData::ExportAssignment(d) => m!(d.modifiers) || o!(d.type_) || n!(d.expression),
        NodeData::NamespaceExportDeclaration(d) => m!(d.modifiers) || n!(d.name),
        NodeData::NamespaceExport(d) => n!(d.name),
        NodeData::NamedExports(d) => l!(d.elements),
        NodeData::ExportSpecifier(d) => o!(d.property_name) || n!(d.name),
        NodeData::CallSignatureDeclaration(d) => {
            ol!(d.type_parameters) || l!(d.parameters) || o!(d.type_)
        }
        NodeData::ConstructSignatureDeclaration(d) => {
            ol!(d.type_parameters) || l!(d.parameters) || o!(d.type_)
        }
        NodeData::ConstructorDeclaration(d) => {
            m!(d.modifiers)
                || ol!(d.type_parameters)
                || l!(d.parameters)
                || o!(d.type_)
                || w.full_signature(d.full_signature)
                || o!(d.body)
        }
        NodeData::GetAccessorDeclaration(d) => {
            m!(d.modifiers)
                || n!(d.name)
                || ol!(d.type_parameters)
                || l!(d.parameters)
                || o!(d.type_)
                || w.full_signature(d.full_signature)
                || o!(d.body)
        }
        NodeData::SetAccessorDeclaration(d) => {
            m!(d.modifiers)
                || n!(d.name)
                || ol!(d.type_parameters)
                || l!(d.parameters)
                || o!(d.type_)
                || w.full_signature(d.full_signature)
                || o!(d.body)
        }
        NodeData::IndexSignatureDeclaration(d) => {
            m!(d.modifiers) || l!(d.parameters) || n!(d.type_)
        }
        NodeData::MethodSignatureDeclaration(d) => {
            m!(d.modifiers)
                || n!(d.name)
                || o!(d.postfix_token)
                || ol!(d.type_parameters)
                || l!(d.parameters)
                || o!(d.type_)
        }
        NodeData::MethodDeclaration(d) => {
            m!(d.modifiers)
                || o!(d.asterisk_token)
                || n!(d.name)
                || o!(d.postfix_token)
                || ol!(d.type_parameters)
                || l!(d.parameters)
                || o!(d.type_)
                || w.full_signature(d.full_signature)
                || o!(d.body)
        }
        NodeData::PropertySignatureDeclaration(d) => {
            m!(d.modifiers) || n!(d.name) || o!(d.postfix_token) || n!(d.type_) || n!(d.initializer)
        }
        NodeData::PropertyDeclaration(d) => {
            m!(d.modifiers) || n!(d.name) || o!(d.postfix_token) || o!(d.type_) || o!(d.initializer)
        }
        NodeData::ClassStaticBlockDeclaration(d) => m!(d.modifiers) || n!(d.body),
        NodeData::BinaryExpression(d) => {
            m!(d.modifiers) || n!(d.left) || o!(d.type_) || n!(d.operator_token) || n!(d.right)
        }
        NodeData::PrefixUnaryExpression(d) => n!(d.operand),
        NodeData::PostfixUnaryExpression(d) => n!(d.operand),
        NodeData::YieldExpression(d) => o!(d.asterisk_token) || o!(d.expression),
        NodeData::ArrowFunction(d) => {
            m!(d.modifiers)
                || ol!(d.type_parameters)
                || l!(d.parameters)
                || o!(d.type_)
                || w.full_signature(d.full_signature)
                || n!(d.equals_greater_than_token)
                || n!(d.body)
        }
        NodeData::FunctionExpression(d) => {
            m!(d.modifiers)
                || o!(d.asterisk_token)
                || o!(d.name)
                || ol!(d.type_parameters)
                || l!(d.parameters)
                || o!(d.type_)
                || w.full_signature(d.full_signature)
                || n!(d.body)
        }
        NodeData::AsExpression(d) => n!(d.expression) || n!(d.type_),
        NodeData::SatisfiesExpression(d) => n!(d.expression) || n!(d.type_),
        NodeData::ConditionalExpression(d) => {
            n!(d.condition)
                || n!(d.question_token)
                || n!(d.when_true)
                || n!(d.colon_token)
                || n!(d.when_false)
        }
        NodeData::PropertyAccessExpression(d) => {
            n!(d.expression) || o!(d.question_dot_token) || n!(d.name)
        }
        NodeData::ElementAccessExpression(d) => {
            n!(d.expression) || o!(d.question_dot_token) || n!(d.argument_expression)
        }
        NodeData::CallExpression(d) => {
            n!(d.expression) || o!(d.question_dot_token) || ol!(d.type_arguments) || l!(d.arguments)
        }
        NodeData::NewExpression(d) => n!(d.expression) || ol!(d.type_arguments) || ol!(d.arguments),
        NodeData::MetaProperty(d) => n!(d.name),
        NodeData::NonNullExpression(d) => n!(d.expression),
        NodeData::SpreadElement(d) => n!(d.expression),
        NodeData::TemplateExpression(d) => n!(d.head) || l!(d.template_spans),
        NodeData::TemplateSpan(d) => n!(d.expression) || n!(d.literal),
        NodeData::TaggedTemplateExpression(d) => {
            n!(d.tag) || o!(d.question_dot_token) || ol!(d.type_arguments) || n!(d.template)
        }
        NodeData::ParenthesizedExpression(d) => n!(d.expression),
        NodeData::ArrayLiteralExpression(d) => l!(d.elements),
        NodeData::ObjectLiteralExpression(d) => l!(d.properties),
        NodeData::SpreadAssignment(d) => n!(d.expression),
        NodeData::PropertyAssignment(d) => {
            m!(d.modifiers) || n!(d.name) || o!(d.postfix_token) || o!(d.type_) || n!(d.initializer)
        }
        NodeData::ShorthandPropertyAssignment(d) => {
            m!(d.modifiers)
                || n!(d.name)
                || o!(d.postfix_token)
                || o!(d.type_)
                || o!(d.equals_token)
                || o!(d.object_assignment_initializer)
        }
        NodeData::DeleteExpression(d) => n!(d.expression),
        NodeData::TypeOfExpression(d) => n!(d.expression),
        NodeData::VoidExpression(d) => n!(d.expression),
        NodeData::AwaitExpression(d) => n!(d.expression),
        NodeData::TypeAssertion(d) => n!(d.type_) || n!(d.expression),
        NodeData::UnionTypeNode(d) => l!(d.types),
        NodeData::IntersectionTypeNode(d) => l!(d.types),
        NodeData::ConditionalTypeNode(d) => {
            n!(d.check_type) || n!(d.extends_type) || n!(d.true_type) || n!(d.false_type)
        }
        NodeData::TypeOperatorNode(d) => n!(d.type_),
        NodeData::InferTypeNode(d) => n!(d.type_parameter),
        NodeData::ArrayTypeNode(d) => n!(d.element_type),
        NodeData::IndexedAccessTypeNode(d) => n!(d.object_type) || n!(d.index_type),
        NodeData::TypeReferenceNode(d) => n!(d.type_name) || ol!(d.type_arguments),
        NodeData::ExpressionWithTypeArguments(d) => n!(d.expression) || ol!(d.type_arguments),
        NodeData::LiteralTypeNode(d) => n!(d.literal),
        NodeData::TypePredicateNode(d) => {
            o!(d.asserts_modifier) || n!(d.parameter_name) || o!(d.type_)
        }
        NodeData::ImportAttribute(d) => n!(d.name) || n!(d.value),
        NodeData::ImportAttributes(d) => l!(d.attributes),
        NodeData::TypeQueryNode(d) => n!(d.expr_name) || ol!(d.type_arguments),
        NodeData::MappedTypeNode(d) => {
            o!(d.readonly_token)
                || n!(d.type_parameter)
                || o!(d.name_type)
                || o!(d.question_token)
                || o!(d.type_)
                || ol!(d.members)
        }
        NodeData::TypeLiteralNode(d) => l!(d.members),
        NodeData::TupleTypeNode(d) => l!(d.elements),
        NodeData::NamedTupleMember(d) => {
            o!(d.dot_dot_dot_token) || n!(d.name) || o!(d.question_token) || n!(d.type_)
        }
        NodeData::OptionalTypeNode(d) => n!(d.type_),
        NodeData::RestTypeNode(d) => n!(d.type_),
        NodeData::ParenthesizedTypeNode(d) => n!(d.type_),
        NodeData::FunctionTypeNode(d) => ol!(d.type_parameters) || l!(d.parameters) || o!(d.type_),
        NodeData::ConstructorTypeNode(d) => {
            m!(d.modifiers) || ol!(d.type_parameters) || l!(d.parameters) || o!(d.type_)
        }
        NodeData::TemplateLiteralTypeNode(d) => n!(d.head) || l!(d.template_spans),
        NodeData::TemplateLiteralTypeSpan(d) => n!(d.type_) || n!(d.literal),
        NodeData::SyntheticExpression(d) => o!(d.tuple_name_source),
        NodeData::PartiallyEmittedExpression(d) => n!(d.expression),
        NodeData::JsxElement(d) => n!(d.opening_element) || l!(d.children) || n!(d.closing_element),
        NodeData::JsxAttributes(d) => l!(d.properties),
        NodeData::JsxNamespacedName(d) => n!(d.namespace) || n!(d.name),
        NodeData::JsxOpeningElement(d) => {
            n!(d.tag_name) || ol!(d.type_arguments) || n!(d.attributes)
        }
        NodeData::JsxSelfClosingElement(d) => {
            n!(d.tag_name) || ol!(d.type_arguments) || n!(d.attributes)
        }
        NodeData::JsxFragment(d) => {
            n!(d.opening_fragment) || l!(d.children) || n!(d.closing_fragment)
        }
        NodeData::JsxAttribute(d) => n!(d.name) || o!(d.initializer),
        NodeData::JsxSpreadAttribute(d) => n!(d.expression),
        NodeData::JsxClosingElement(d) => n!(d.tag_name),
        NodeData::JsxExpression(d) => o!(d.dot_dot_dot_token) || o!(d.expression),
        NodeData::SyntaxList(d) => w.ids(&d.children),
        NodeData::JsDoc(d) => l!(d.comment) || ol!(d.tags),
        NodeData::JsDocTypeExpression(d) => n!(d.type_),
        NodeData::JsDocNonNullableType(d) => n!(d.type_),
        NodeData::JsDocNullableType(d) => n!(d.type_),
        NodeData::JsDocVariadicType(d) => n!(d.type_),
        NodeData::JsDocOptionalType(d) => n!(d.type_),
        NodeData::JsDocTypeTag(d) => n!(d.tag_name) || n!(d.type_expression) || ol!(d.comment),
        NodeData::JsDocReturnTag(d) => n!(d.tag_name) || o!(d.type_expression) || ol!(d.comment),
        NodeData::JsDocSatisfiesTag(d) => n!(d.tag_name) || n!(d.type_expression) || ol!(d.comment),
        NodeData::JsDocThrowsTag(d) => n!(d.tag_name) || o!(d.type_expression) || ol!(d.comment),
        NodeData::JsDocThisTag(d) => n!(d.tag_name) || n!(d.type_expression) || ol!(d.comment),
        NodeData::JsDocOverloadTag(d) => n!(d.tag_name) || n!(d.type_expression) || ol!(d.comment),
        NodeData::JsDocUnknownTag(d) => n!(d.tag_name) || ol!(d.comment),
        NodeData::JsDocPublicTag(d) => n!(d.tag_name) || ol!(d.comment),
        NodeData::JsDocPrivateTag(d) => n!(d.tag_name) || ol!(d.comment),
        NodeData::JsDocProtectedTag(d) => n!(d.tag_name) || ol!(d.comment),
        NodeData::JsDocReadonlyTag(d) => n!(d.tag_name) || ol!(d.comment),
        NodeData::JsDocOverrideTag(d) => n!(d.tag_name) || ol!(d.comment),
        NodeData::JsDocDeprecatedTag(d) => n!(d.tag_name) || ol!(d.comment),
        NodeData::JsDocTemplateTag(d) => {
            n!(d.tag_name) || n!(d.constraint) || l!(d.type_parameters) || ol!(d.comment)
        }
        NodeData::JsDocSeeTag(d) => n!(d.tag_name) || n!(d.name_expression) || ol!(d.comment),
        NodeData::JsDocImplementsTag(d) => n!(d.tag_name) || n!(d.class_name) || ol!(d.comment),
        NodeData::JsDocAugmentsTag(d) => n!(d.tag_name) || n!(d.class_name) || ol!(d.comment),
        NodeData::JsDocImportTag(d) => {
            n!(d.tag_name)
                || o!(d.import_clause)
                || n!(d.module_specifier)
                || o!(d.attributes)
                || ol!(d.comment)
        }
        NodeData::JsDocCallbackTag(d) => {
            n!(d.tag_name) || n!(d.type_expression) || o!(d.name) || ol!(d.comment)
        }
        NodeData::JsDocTypedefTag(d) => {
            n!(d.tag_name) || o!(d.type_expression) || o!(d.name) || ol!(d.comment)
        }
        NodeData::JsDocSignature(d) => ol!(d.type_parameters) || l!(d.parameters) || o!(d.type_),
        NodeData::JsDocNameReference(d) => n!(d.name),
        // Go: ast.go:3170 forEachChild_JSDocParameterOrPropertyTag
        NodeData::JsDocParameterOrPropertyTag(d) => {
            n!(d.tag_name)
                || (d.is_name_first && (n!(d.name) || o!(d.type_expression)))
                || (!d.is_name_first && (o!(d.type_expression) || n!(d.name)))
                || ol!(d.comment)
        }
        NodeData::ModuleDeclaration(d) => {
            m!(d.modifiers) || n!(d.name) || o!(d.attributes) || o!(d.body)
        }
        NodeData::ImportEqualsDeclaration(d) => {
            m!(d.modifiers) || n!(d.name) || n!(d.module_reference)
        }
        NodeData::ExportDeclaration(d) => {
            m!(d.modifiers) || o!(d.export_clause) || o!(d.module_specifier) || o!(d.attributes)
        }
        NodeData::ImportTypeNode(d) => {
            n!(d.argument) || o!(d.attributes) || o!(d.qualifier) || ol!(d.type_arguments)
        }
        NodeData::ImportClause(d) => o!(d.name) || o!(d.named_bindings),
        NodeData::ImportSpecifier(d) => o!(d.property_name) || n!(d.name),
        NodeData::JsDocLink(d) => o!(d.name),
        NodeData::JsDocLinkCode(d) => o!(d.name),
        NodeData::JsDocLinkPlain(d) => o!(d.name),
        NodeData::TypeParameterDeclaration(d) => {
            m!(d.modifiers)
                || n!(d.name)
                || o!(d.constraint)
                || o!(d.expression)
                || o!(d.default_type)
        }
        NodeData::SyntheticReferenceExpression(d) => n!(d.expression) || n!(d.this_arg),
        NodeData::JsDocTypeLiteral(d) => match &d.js_doc_property_tags {
            Some(tags) => w.ids(tags),
            None => false,
        },
        NodeData::SourceFile(d) => l!(d.statements) || n!(d.end_of_file_token),
        _ => false,
    }
}

// ──────────────────────────────────────────────────────────────────────
// Subtree facts (ast.go computeSubtreeFacts / propagateSubtreeFacts)
// ──────────────────────────────────────────────────────────────────────

// Go: subtreefacts.go SubtreeExclusions*
const EXCL_NODE: SubtreeFacts = SubtreeFacts::COMPUTED;

const TS: SubtreeFacts = SubtreeFacts::SUBTREE_CONTAINS_TYPE_SCRIPT;
const NONE_FACTS: SubtreeFacts = SubtreeFacts::NONE;

/// Go `core.IfElse(cond, facts, SubtreeFactsNone)`.
fn facts_if(cond: bool, facts: SubtreeFacts) -> SubtreeFacts {
    if cond { facts } else { NONE_FACTS }
}

// Go: subtreefacts.go:89 propagateEraseableSyntaxListSubtreeFacts
fn propagate_eraseable_list(children: DataList<'_>) -> SubtreeFacts {
    facts_if(children.is_some(), TS)
}

// Go: subtreefacts.go:93 propagateEraseableSyntaxSubtreeFacts
fn propagate_eraseable(child: Node) -> SubtreeFacts {
    facts_if(child.is_some(), TS)
}

// Go: subtreefacts.go:97 propagateObjectBindingElementSubtreeFacts

// Go: subtreefacts.go:106 propagateBindingElementSubtreeFacts

// Go: subtreefacts.go:110 propagateSubtreeFacts

// Go: subtreefacts.go:117 propagateNodeListSubtreeFacts
// Go: subtreefacts.go:128 propagateModifierListSubtreeFacts (with
// `propagate_subtree_facts`)
// PORT: the list is read in place from the node data (`DataList`).



// Go: ast.go:1246 (*NodeDefault).propagateSubtreeFacts and the per-type
// propagateSubtreeFacts overrides in ast.go.
// PORT: the Go overrides are merged into one match on the node data.
// PERF: emitast1 F2. One data read per child: the type syntax test, the
// facts of a child that has none cached yet and the exclusion all use it.


// Go: ast.go:1602 (*CompositeBase).subtreeFactsWorker and ast.go:1234
// (*NodeDefault).subtreeFactsWorker



// Go: ast.go computeSubtreeFacts overrides (ast.go:1619 to ast.go:2689) and
// ast_generated.go computeSubtreeFacts methods.
// PORT: the Go per-type methods are merged into one match. Types without an
// override use Go `(*NodeDefault).computeSubtreeFacts`, which is None.
// `(*TypeSyntaxBase).computeSubtreeFacts` is in `subtree_facts_with_data`.

/// Go `node.modifiers != nil && node.modifiers.ModifierFlags&ModifierFlagsAsync != 0`
/// for the modifier list `m` of a node, read in place.
fn modifiers_have_async(m: &Option<crate::astdata::ModifierList>) -> bool {
    in_place_modifier_flags(m).intersects(ModifierFlags::ASYNC)
}

// ──────────────────────────────────────────────────────────────────────
// Access kinds
// ──────────────────────────────────────────────────────────────────────

// Go: ast.go:1268 IsWriteOnlyAccess

// Go: ast.go:1272 IsWriteAccess

// Go: ast.go:1276 IsWriteAccessForReference

// Go: ast.go:1281 GetDeclarationFromName

// Go: ast.go:1327 declarationIsWriteAccess

// Go: ast.go:1406 IsArrayLiteralOrObjectLiteralDestructuringPattern

// Go: ast.go:1430 accessKind

// Go: ast.go:1491 reverseAccessKind
fn reverse_access_kind(a: AccessKind) -> AccessKind {
    if a == AccessKind::READ {
        AccessKind::WRITE
    } else if a == AccessKind::WRITE {
        AccessKind::READ
    } else if a == AccessKind::READ_WRITE {
        AccessKind::READ_WRITE
    } else {
        panic!("Unhandled case in reverseAccessKind")
    }
}

// Go: ast.go:1515 IsDeclarationNode

// Go: ast.go:1532 IsLocalsContainer

impl Node {
    // Go: ast.go:1563 JSDoc

    // Go: ast.go:1581 EagerJSDoc
}


// Go: ast.go:1832 IsTypeOrJSTypeAliasDeclaration

// Go: ast.go:1878 IsImportDeclarationOrJSImportDeclaration

// Go: ast.go:1904 IsAnyExportAssignment

impl Node {
    // Go: ast.go:2217 (*ImportAttributesNode).GetResolutionModeOverride
}

// ──────────────────────────────────────────────────────────────────────
// SourceFile methods. `file` is a SourceFile node.
// ──────────────────────────────────────────────────────────────────────

// PORT: the caches below are per thread and leaked, for static files
// (static publishes, synthetic) and files that are not published yet. A
// published freeable file version (lsshells M3b) keeps them on its
// `FileVersion` (`name_table`, `position_map`, `declaration_map`; see
// `file_version_of`) and its ECMA line map in its store
// (`frozen_file_ecma_line_starts`), as Go keeps them on the `SourceFile`,
// so they are freed with it.
thread_local! {
    /// Go `SourceFile.ecmaLineMap`, computed once per file.
    static ECMA_LINE_MAPS: RefCell<FxHashMap<Node, &'static [i32]>> = RefCell::new(FxHashMap::default());
    /// Go `SourceFile.nameTable`, computed once per file.
    static NAME_TABLES: RefCell<FxHashMap<Node, &'static FxHashMap<String, i32>>> = RefCell::new(FxHashMap::default());
    /// Go `SourceFile.positionMap`, computed once per file.
    static POSITION_MAPS: RefCell<FxHashMap<Node, &'static PositionMap>> = RefCell::new(FxHashMap::default());
    /// Go `SourceFile.declarationMap`, computed once per file.
    static DECLARATION_MAPS: RefCell<FxHashMap<Node, &'static FxHashMap<String, Vec<Node>>>> =
        RefCell::new(FxHashMap::default());
    /// Go `SourceFile.identifiers`, collected once per parsed file.
    static IDENTIFIER_SETS: RefCell<FxHashMap<Node, &'static FxHashSet<&'static str>>> =
        RefCell::new(FxHashMap::default());
}


// Go: ast.go:2566 (*SourceFile).Text

// ---------------------------------------------------------------------------
// Content mapper info of a SourceFile (tsgo#4712)
// ---------------------------------------------------------------------------

// Go: ast/ast.go:2602 MappedDiagnosticDirectivePolicy
crate::flags_macros::go_enum!(MappedDiagnosticDirectivePolicy, u8 {
    IGNORE = 0; // MappedDiagnosticDirectivePolicyIgnore
    EXPECT = 1; // MappedDiagnosticDirectivePolicyExpect
});

// Go: ast/ast.go:2609 MappedDiagnosticDirective
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MappedDiagnosticDirective {
    pub original_range: TextRange,
    pub virtual_range: TextRange,
    pub policy: MappedDiagnosticDirectivePolicy,
    pub unused_code: i32,
    pub unused_message_text: String,
    pub source: String,
}

// Go: ast/ast.go:2618 ContentMapperSourceFileInfo
// PORT: Go `*SourceFile` is `Rc<ParsedSourceFile>` (the compiler host
// type), and Go `*spanmap.SpanMap` is `Option<Arc<SpanMap>>`. Pass it to
// `ParsedSourceFile::set_content_mapper_info`, which keeps it in two parts:
// `ContentMapperFileInfo` for readers of the SourceFile node on any thread,
// and the `Rc` links on this thread.

/// Go `SourceFile.contentMapperInfo` as the readers of a SourceFile node
/// see it: `ContentMapperSourceFileInfo` with each linked file as its
/// SourceFile node (`Node::NIL` for no canonical file).
// PORT: Go keeps the info on the SourceFile and reads it on any goroutine
// (the emitter, the diagnostic writer, the language service). A parsed
// file's node data cannot hold it and `ParsedSourceFile` must stay `Send`
// (a parse worker makes it), so the info lives in a process table by file
// (`content_mapper_key`), like the file stores. It is set once and never
// freed, as a published file is never freed.
#[derive(Debug)]
pub struct ContentMapperFileInfo {
    pub content_mapper: String,
    pub transform_identity: String,
    pub parse_options: crate::frontend::parser::SourceFileParseOptions,
    pub virtual_file_name: String,
    pub original_text: String,
    pub span_map: Option<std::sync::Arc<crate::spanmap::SpanMap>>,
    pub diagnostic_directives: Vec<MappedDiagnosticDirective>,
    pub supplemental_source_files: Vec<Node>,
    pub canonical_source_file: Node,
}



/// True once any file has content mapper info, so a program with no content
/// mapper reads no table.
static HAS_CONTENT_MAPPER_INFO: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Go `SourceFileParseOptions{}`, which `ContentMapperParseOptions` returns
/// for a file with no content mapper info.
static EMPTY_PARSE_OPTIONS: crate::frontend::parser::SourceFileParseOptions =
    crate::frontend::parser::SourceFileParseOptions {
        file_name: String::new(),
        path: crate::frontend::tspath::Path(String::new()),
        external_module_indicator_options:
            crate::frontend::parser::ExternalModuleIndicatorOptions {
                jsx: false,
                force: false,
            },
    };

// Go: ast/ast.go:2639 (*SourceFile).SetContentMapperInfo, for the node side.


// Go: ast/ast.go:2548 (*SourceFile).OriginalText
// OriginalText returns the untransformed source text for content-mapped files, or Text() otherwise.

// Go: ast/ast.go:2556 (*SourceFile).OriginalFileName
// OriginalFileName returns the canonical filename associated with a supplemental source file, or FileName() otherwise.

// Go: ast/ast.go:2566 (*SourceFile).SpanMap
// SpanMap returns the span map that maps positions in this file's transformed Text() back to its
// original, untransformed content, or nil if the file is not content-mapped (or is a failure stub).
// The returned map is nil-safe: a nil map maps positions identically.
// PORT: nil is `None`; the `SpanMap` methods take `Option<&SpanMap>`.

// Go: ast/ast.go:2575 (*SourceFile).ContentMapper
// ContentMapper returns the identity of the content mapper that produced this file, or "" if the file
// was not produced by a content mapper (or the mapper did not identify itself).

// Go: ast/ast.go:2584 (*SourceFile).IsContentMapperFailureStub
// IsContentMapperFailureStub reports whether this file is the empty placeholder produced when a content
// mapper's transform failed.

// Go: ast/ast.go:2588 (*SourceFile).ContentMapperTransformIdentity

// Go: ast/ast.go:2595 (*SourceFile).VirtualFileName

// Go: ast/ast.go:2631 (*SourceFile).ContentMapperParseOptions
// ContentMapperParseOptions returns the parse options used to acquire this file from the mapped parse cache.
// PORT: returns a reference; the zero value is `EMPTY_PARSE_OPTIONS`.

// Go: ast/ast.go:2646 (*SourceFile).DiagnosticDirectives

// Go: ast/ast.go:2654 (*SourceFile).SupplementalSourceFiles
// SupplementalSourceFiles returns the additional outputs produced from this canonical source file.
// PORT: the files are their SourceFile nodes. `ParsedSourceFile` has the
// `Rc` form.

// Go: ast/ast.go:2662 (*SourceFile).CanonicalSourceFile
// CanonicalSourceFile returns the canonical output associated with this supplemental source file.
// PORT: the file is its SourceFile node, or `Node::NIL`. `ParsedSourceFile`
// has the `Rc` form.

// Go: ast/ast.go:2670 (*SourceFile).IsContentMapperSupplemental
// IsContentMapperSupplemental reports whether this is an unnamed supplemental mapper output.

// Go: ast.go:2674 (*SourceFile).HasIdentifier
// PORT: Go `identifiersOnce` is a per-thread cache (`IDENTIFIER_SETS`) for a
// parsed file, whose id is never reused. A freeable file version keeps its
// set, which is freed with it (lsshells M3). A factory SourceFile collects
// its set on each call, because a released program frees its synthetic
// nodes and a later node can get the same handle.

// Go: ast.go:2681 collectIdentifiersForSourceFile

// Go: ast.go:2570 (*SourceFile).FileName

// Go: ast.go:2578 (*SourceFile).Imports
// PORT: a freeable file version (lsshells M3b) gives a handle slice
// (`NodeSlice::from_file_imports`), which reads the imports at each use.

/// A diagnostic list of `SourceFileInfo`, as the key of a `FileRef`.
#[derive(Clone, Copy)]
enum DiagnosticList {
    Parse,
    Js,
    JsDoc,
}



// Go: ast.go:2582 (*SourceFile).Diagnostics
// PORT: a file that is not published yet reads its store (see
// `source_file_language_variant`).

// Go: ast.go:2590 (*SourceFile).JSDiagnostics

// Go: ast.go:2598 (*SourceFile).JSDocDiagnostics

// Go: ast.go:2641 (*SourceFile).BindDiagnostics
// PORT: panics before the file is bound. Go returns nil there.

// Go: ast.go:2657 (*SourceFile).IsJS

// Go: ast.go:2702 (*SourceFile).ECMALineMap

// Go: ast.go:2720 (*SourceFile).GetNameTable


// Go: ast.go:2751 (*SourceFile).IsBound

// Go: ast.go:2756 (*SourceFile).GetPositionMap
// PORT: Go `positionMapOnce` is a per-thread cache (`POSITION_MAPS`).


// Go: ast.go:2840 (*SourceFile).GetDeclarationMap

// Go: ast.go:2849 (*SourceFile).computeDeclarationMap

// Go: ast.go:3087 GetDeclarationName

#[cfg(test)]
mod tests {
    use super::*;

    /// Go `(*Node)(nil).SubtreeFacts()` dereferences nil. The data read of
    /// nil panics (`static_tier_ast_node`) before any facts cache read, in a
    /// debug build as in a release build.
    #[test]
    #[should_panic(expected = "nil node dereference")]
    fn subtree_facts_of_nil_is_a_nil_dereference() {
        let _ = Node::NIL.subtree_facts();
    }

    /// Go caches `SubtreeFacts()` only in nodes of a `CompositeBase` type
    /// (`ast.go:1602`). Tokens, identifiers, literals and type syntax compute
    /// their facts on each call (`ast.go:1234`), with the same values.
    #[test]
    fn subtree_facts_are_cached_only_for_composite_kinds() {
        use crate::frontend::parser::{SourceFileParseOptions, parse_source_file};
        fn collect(n: Node, out: &mut Vec<Node>) {
            out.push(n);
            n.for_each_child(|c| {
                collect(c, out);
                false
            });
        }
        let text = "@dec class C { m(@p a: number = 1) { return a ?? this.b?.c; } }\n\
            let x: string[] = [1, 2].map(y => `${y}`), z = !(-x.length);\n";
        let opts = SourceFileParseOptions {
            file_name: "/facts.ts".to_string(),
            ..Default::default()
        };
        let root = parse_source_file(&opts, text, ScriptKind::TS).root;
        assert!(
            root.subtree_facts()
                .intersects(SubtreeFacts::SUBTREE_CONTAINS_DECORATORS)
        );
        let mut nodes = Vec::new();
        collect(root, &mut nodes);
        let (mut cached, mut computed) = (0, 0);
        for &n in &nodes {
            let facts = n.subtree_facts();
            assert_eq!(n.subtree_facts(), facts, "{:?}", n.kind());
            if with_data!(n, |d| is_composite_data(n, d)) {
                assert_eq!(cached_subtree_facts(n), Some(facts), "{:?}", n.kind());
                cached += 1;
            } else {
                assert_eq!(cached_subtree_facts(n), None, "{:?}", n.kind());
                computed += 1;
            }
        }
        assert!(cached > 10 && computed > 10, "{cached} {computed}");
    }
}
