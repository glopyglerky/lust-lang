//! Port of typescript-go `internal/parser/reparser.go`: JSDoc tags in JS
//! files become synthetic TypeScript nodes (unit U9).
//!
//! Hosted tags find a host node and add their children to it. Unhosted tags
//! add new statements to the reparse list.
//!
//! PORT: Go writes node fields in place (`node.AsX().Field = v`,
//! `AsMutable().SetType(..)`). Here each write clones the node data, changes
//! the field and calls `replace_node_data` (see `mutate`). Go list appends
//! (`list.Nodes = append(list.Nodes, n)`) make a new list with the same
//! `Loc`, because Rust lists cannot change.
//!
//! PORT: Go `p.nodeSliceArena` only saves allocations. Here each call uses a
//! new `Vec`.

use crate::astdata::NodeData as D;
use crate::frontend::prelude::*;


/// Runs `$body` with `$d` bound to the data of a function-like node (Go
/// `FunctionLikeData()`). `$index_body` is for IndexSignatureDeclaration,
/// whose `type_` field is required in astdata. Other kinds panic, like a Go
/// nil `FunctionLikeData()` dereference.
macro_rules! with_function_like_data {
    ($data:expr, $d:ident => $body:expr, index $i:ident => $index_body:expr) => {
        match $data {
            D::ArrowFunction($d) => $body,
            D::CallSignatureDeclaration($d) => $body,
            D::ConstructSignatureDeclaration($d) => $body,
            D::ConstructorDeclaration($d) => $body,
            D::ConstructorTypeNode($d) => $body,
            D::FunctionDeclaration($d) => $body,
            D::FunctionExpression($d) => $body,
            D::FunctionTypeNode($d) => $body,
            D::GetAccessorDeclaration($d) => $body,
            D::JsDocSignature($d) => $body,
            D::MethodDeclaration($d) => $body,
            D::MethodSignatureDeclaration($d) => $body,
            D::SetAccessorDeclaration($d) => $body,
            D::IndexSignatureDeclaration($i) => $index_body,
            other => panic!("node data is not function-like: {}", other.schema_name()),
        }
    };
}

// Go: ast/ast.go:677 (m *MutableNode) SetType
// PORT: only the kinds that the reparser writes. Go panics on other kinds.

// Go: ast/ast.go:387 (m *MutableNode) SetExpression
// PORT: only the kinds that the reparser writes.

// Go: ast/ast.go:765 (m *MutableNode) SetInitializer
// PORT: only the kinds that the reparser writes.

// Go: ast/ast.go:227 (m *MutableNode) SetModifiers
// PORT: only the kinds that the reparser writes.






// Go: parser/reparser.go:625 findMatchingParameter
// PORT: returns the parameter node, or nil for Go `(nil, false)`.

// Go: parser/reparser.go:650 skipSatisfiesExpressions

// Go: parser/reparser.go:657 getFunctionLikeHost

// Go: parser/reparser.go:689 getClassLikeData
// PORT: Go returns the `*ClassLikeBase` data. Here the class node (or nil);
// callers read and write its fields through the node.
