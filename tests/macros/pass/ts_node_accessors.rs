// macro: sel_field, variant_has_kind, kind_lacks_data, match_data, by_data, list_by_data, data_accessor, modifiers_variants, modifiers_accessor, store_node_modifier_bits_fn, name_arms, expression_arms, postfix_arms, question_arms, type_arms, initializer_arms, type_arguments_arms, initializer_data_accessor, other_column_match, name_data_accessor, postfix_token_data_accessor, own_question_token_accessor, locals_container_variants, data_is_variant, with_data, list_of, modifiers_of
// source: crates/ts_goport/src/ast/node.rs:39
// source: crates/ts_goport/src/ast/node.rs:145
// source: crates/ts_goport/src/ast/node.rs:299
// source: crates/ts_goport/src/ast/node.rs:310
// source: crates/ts_goport/src/ast/node.rs:330
// source: crates/ts_goport/src/ast/node.rs:348
// source: crates/ts_goport/src/ast/node.rs:379
// source: crates/ts_goport/src/ast/node.rs:411
// source: crates/ts_goport/src/ast/node.rs:458
// source: crates/ts_goport/src/ast/node.rs:484
// source: crates/ts_goport/src/ast/node.rs:520
// source: crates/ts_goport/src/ast/node.rs:576
// source: crates/ts_goport/src/ast/node.rs:625
// source: crates/ts_goport/src/ast/node.rs:646
// source: crates/ts_goport/src/ast/node.rs:668
// source: crates/ts_goport/src/ast/node.rs:721
// source: crates/ts_goport/src/ast/node.rs:745
// source: crates/ts_goport/src/ast/node.rs:766
// source: crates/ts_goport/src/ast/node.rs:782
// source: crates/ts_goport/src/ast/node.rs:811
// source: crates/ts_goport/src/ast/node.rs:830
// source: crates/ts_goport/src/ast/node.rs:845
// source: crates/ts_goport/src/ast/node.rs:968
// source: crates/ts_goport/src/ast/node.rs:1006
// source: crates/ts_goport/src/ast/synthetic.rs:1341
// source: crates/ts_goport/src/ast/synthetic.rs:1364
// source: crates/ts_goport/src/ast/synthetic.rs:1389
// The macro_rules! text below is copied exact from ts-rust. Only the
// surrounding types are minimal stand-ins. See tests/macros/README.md.
use crate::ast::synthetic::AnyList;

include!("node_scaffold.rs.inc");

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

macro_rules! kind_lacks_data {
    ($n:expr, [$($v:ident),+]) => {
        match frozen_store_kind($n) {
            Some(kind) => kind != SyntaxKind::Unknown $(&& !variant_has_kind!($v, kind))+,
            None => false,
        }
    };
}

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

macro_rules! data_is_variant {
    ($d:expr, [$($v:ident),+ $(,)?]) => {
        matches!($d, $(NodeData::$v(_))|+)
    };
}

macro_rules! with_data {
    ($n:expr, |$d:ident| $body:expr) => {{
        let n__: $crate::core::Node = $n;
        match $crate::ast::synthetic::static_tier_ast_node(n__) {
            Some(data__) => {
                let $d: &crate::astdata::NodeData = data__;
                $body
            }
            None => $crate::ast::synthetic::read_ast_node_miss(n__, |data__| {
                let $d: &crate::astdata::NodeData = data__;
                $body
            }),
        }
    }};
}

macro_rules! list_of {
    ($n:expr, |$d:ident| $body:expr) => {{
        let n__: $crate::core::Node = $n;
        match $crate::ast::synthetic::static_tier_ast_node(n__) {
            Some(data__) => {
                let $d: &'static crate::astdata::NodeData = data__;
                let found: Option<Option<$crate::ast::synthetic::AnyList<'static>>> = $body;
                found.map(|l| {
                    $crate::ast::NodeList::from_ts(
                        n__.file_index(),
                        l.map($crate::ast::synthetic::AnyList::nodes),
                    )
                })
            }
            None => {
                static SITE: $crate::ast::synthetic::SelectorSite =
                    $crate::ast::synthetic::SelectorSite::new();
                $crate::ast::synthetic::scoped_node_list_of(n__, &SITE, |$d| $body)
            }
        }
    }};
}

macro_rules! modifiers_of {
    ($n:expr, |$d:ident| $body:expr) => {{
        let n__: $crate::core::Node = $n;
        match $crate::ast::synthetic::static_tier_ast_node(n__) {
            Some(data__) => {
                let $d: &'static crate::astdata::NodeData = data__;
                let found: Option<Option<$crate::ast::synthetic::AnyList<'static>>> = $body;
                found.map(|m| {
                    $crate::ast::ModifierList::from_ts(
                        n__.file_index(),
                        m.map($crate::ast::synthetic::AnyList::modifiers),
                    )
                })
            }
            None => {
                static SITE: $crate::ast::synthetic::SelectorSite =
                    $crate::ast::synthetic::SelectorSite::new();
                $crate::ast::synthetic::scoped_modifiers_of(n__, &SITE, |$d| $body)
            }
        }
    }};
}

fn sel_type_arguments(_file: usize, l: &Option<crate::astdata::NodeList>) -> Option<Option<AnyList<'_>>> {
    Some(l.as_ref().map(AnyList::Nodes))
}

modifiers_variants!(store_node_modifier_bits_fn! {});

pub fn is_locals_container(node: Node) -> bool {
    with_data!(node, |d| locals_container_variants!(data_is_variant! { d, }))
}

// The simplified SlotChildren read: the same arm-macro plumbing as
// ts-rust store_node_children_from_data, over the stub column type.
fn store_children(data: &NodeData) -> SlotChildren {
    let id_opt = |_: usize, id: Option<crate::astdata::NodeId>| Some(id.map_or(0, |i| i.0));
    let id_req = |_: usize, id: crate::astdata::NodeId| Some(id.0);
    let name = name_arms!(match_data!(0, data, Some(0), [QualifiedName] => |_f, _d| None,), id_opt, id_req);
    let tagged_opt = |tag: u32| move |_: usize, id: Option<crate::astdata::NodeId>| Some((tag, id.map_or(0, |i| i.0)));
    let tagged_req = |tag: u32| move |_: usize, id: crate::astdata::NodeId| Some((tag, id.0));
    let (expr_opt, expr_req) = (tagged_opt(SlotChildren::TAG_EXPRESSION), tagged_req(SlotChildren::TAG_EXPRESSION));
    let postfix = tagged_opt(SlotChildren::TAG_POSTFIX);
    let (q_opt, q_req) = (tagged_opt(SlotChildren::TAG_QUESTION), tagged_req(SlotChildren::TAG_QUESTION));
    let other = other_column_match!(@expression data, expr_opt, expr_req, postfix, q_opt, q_req);
    let type_id = type_arms!(match_data!(0, data, None,), id_opt, id_req);
    let typed = type_id.map(|t| (SlotChildren::TYPED_TYPE, t));
    let list_is_none = |_: usize, list: &Option<crate::astdata::NodeList>| list.is_none();
    let no_type_arguments = type_arguments_arms!(match_data!(0, data, true,), list_is_none);
    SlotChildren::new(name, other).with_typed(typed, no_type_arguments)
}

impl Node {
    name_arms!(name_data_accessor!(), opt, req);
    modifiers_variants!(modifiers_accessor! {});
    initializer_arms!(initializer_data_accessor!(), opt, req);
    postfix_arms!(postfix_token_data_accessor!(), opt);
    question_arms!(own_question_token_accessor!(), some_opt, some_req);

    pub fn parameter_list(self) -> NodeList {
        list_by_data!(
            self, list_of, NodeList::NIL,
            [ArrowFunction, CallSignatureDeclaration, ConstructSignatureDeclaration,
             ConstructorDeclaration, FunctionDeclaration, FunctionExpression,
             GetAccessorDeclaration, SetAccessorDeclaration, IndexSignatureDeclaration,
             JsDocSignature, MethodDeclaration, MethodSignatureDeclaration,
             FunctionTypeNode, ConstructorTypeNode] => req parameters,
        )
    }

    pub fn type_argument_list(self) -> NodeList {
        list_of!(self, |d| type_arguments_arms!(match_data!(0, d, None,), sel_type_arguments))
            .unwrap_or(NodeList::NIL)
    }

    pub fn question_dot_token(self) -> Node {
        by_data!(
            self, Node::NIL,
            [ElementAccessExpression, PropertyAccessExpression, CallExpression,
             TaggedTemplateExpression] => |f, d| opt(f, d.question_dot_token),
        )
    }

    pub fn locals_rank(self) -> u32 {
        if locals_container_variants!(kind_lacks_data! { self, }) { 0 } else { 1 }
    }
}

fn id(n: u32) -> crate::astdata::NodeId { crate::astdata::NodeId(n) }

fn main() {
    let fd = register(0, NodeData::FunctionDeclaration(Box::new(FunctionDeclarationData {
        name: Some(id(5)),
        modifiers: Some(crate::astdata::ModifierList {
            list: crate::astdata::NodeList { items: vec![id(1)] },
            flags: crate::astdata::ModFlags(3),
        }),
        parameters: crate::astdata::NodeList { items: vec![id(2), id(3)] },
        ..Default::default()
    })));
    let vd = register(0, NodeData::VariableDeclaration(Box::new(VariableDeclarationData {
        name: id(6), initializer: Some(id(9)), type_: Some(id(4)), ..Default::default()
    })));
    let call = register(0, NodeData::CallExpression(Box::new(CallExpressionData {
        expression: id(11),
        type_arguments: Some(crate::astdata::NodeList { items: vec![id(1)] }),
        question_dot_token: Some(id(12)),
        ..Default::default()
    })));
    let pd = register(0, NodeData::ParameterDeclaration(Box::new(ParameterDeclarationData {
        name: id(7), question_token: Some(id(13)), ..Default::default()
    })));
    let md = register(0, NodeData::MethodDeclaration(Box::new(MethodDeclarationData {
        name: id(8), postfix_token: Some(id(14)), ..Default::default()
    })));

    println!("name fd={} vd={} in={}", fd.name_data().raw(), vd.name_data().raw(), fd.name_data_in(static_ast_node(fd)).raw());
    println!("init vd={} fd={}", vd.initializer_data().raw(), fd.initializer_data().raw());
    println!("postfix md={} question pd={:?} qdot={}", md.postfix_token_data().raw(), pd.own_question_token().map(|n| n.raw()), call.question_dot_token().raw());
    println!("mods bits={} in_bits={} store_bits={}", fd.modifiers().bits(), fd.modifiers_in(static_ast_node(fd)).bits(),
        store_node_modifier_bits(fd.kind(), static_ast_node(fd).unwrap()));
    println!("params={} typeargs={} typeargs_nil={}", fd.parameter_list().len(), call.type_argument_list().len(), vd.type_argument_list().is_nil());
    println!("locals fd={} vd={} rank={} {}", is_locals_container(fd), is_locals_container(vd), fd.locals_rank(), call.locals_rank());
    let sc = store_children(static_ast_node(vd).unwrap());
    println!("children name={:?} other={:?} typed={:?} no_tyargs={}", sc.name, sc.other, sc.typed, sc.no_type_arguments);
    let k = SyntaxKind::FunctionType;
    println!("has_kind special={} generic={} token={}", variant_has_kind!(FunctionTypeNode, k),
        variant_has_kind!(VariableDeclaration, vd.kind()), variant_has_kind!(Token, SyntaxKind::Identifier));
    println!("nil name={} init={}", Node::NIL.name_data().raw(), Node::NIL.initializer_data().raw());
}
