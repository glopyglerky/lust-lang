//! Port of Go `transformers/transformer.go` plus the shared transformer
//! contract (`TransformOptions`, `TransformerFactory`) from `chain.go`.
//!
//! PORT: Go `Transformer` is a struct that each transformer embeds, with a
//! root visitor built by `NewTransformer(tx.visit, emitContext)`. Here each
//! transformer is its own struct that implements the `Transformer` trait. The
//! root visitor is built on demand (see `TransformerVisit::with_visitor`), so
//! the visit callback can reach the transformer state through `v.ctx`.

use crate::prelude::*;


// Go: binder/referenceresolver.go:9 ReferenceResolver
/// Go `binder.ReferenceResolver` as the transformers see it. The checker
/// `EmitResolver` and the binder resolver both implement it.
// PORT: the binder `ReferenceResolver` struct takes a checker on each call.
// Transformers do not hold a checker, so they use this `&self` interface.
pub trait TransformReferenceResolver {
    fn get_referenced_export_container(&self, node: Node, prefix_locals: bool) -> Node;
    fn get_referenced_import_declaration(&self, node: Node) -> Node;
    fn get_referenced_value_declaration(&self, node: Node) -> Node;
    fn get_referenced_value_declarations(&self, node: Node) -> Vec<Node>;
    fn get_element_access_expression_name(&self, expression: Node) -> String;
    fn get_referenced_member_value_declaration(&self, node: Node) -> Node;
}



// Go: transformers/chain.go:26 TransformOptions
// PORT: Go `GetEmitModuleFormatOfFile func(file ast.HasFileName)` takes the
// source file node here.

// Go: transformers/transformer.go:8 Transformer


// Go: transformers/chain.go:34 TransformerFactory

