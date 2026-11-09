//! Port of Go `transformers/estransforms/utilities.go`, plus the visitor
//! plumbing that every transformer in this package shares.

use super::async_::{};
use crate::prelude::*;
use crate::printer::{AutoGenerateOptions, };


/// Implements `TxVisitors` and the contract `Transformer` for a transformer
/// struct with an `emit_context: Rc<EmitContext>` field and an inherent
/// `fn visit(&mut self, node: Node) -> Node` (Go `tx.visit`).
macro_rules! impl_es_transformer {
    ($t:ty) => {
        impl $crate::transformers::estransforms::utilities::TxVisitors for $t {
            fn ec(&self) -> Rc<$crate::printer::EmitContext> {
                self.emit_context.clone()
            }

            fn root_visit(&mut self, node: Node) -> Node {
                <$t>::visit(self, node)
            }
        }

        impl $crate::transformers::estransforms::contract::Transformer for $t {
            fn emit_context(&self) -> &Rc<$crate::printer::EmitContext> {
                &self.emit_context
            }

            fn transform_source_file(&mut self, file: Node) -> Node {
                $crate::transformers::estransforms::utilities::TxVisitors::transform_source_file_impl(
                    self, file,
                )
            }
        }
    };
}




/// Go `^flags` for a `ModifierFlags` mask.
pub(crate) fn all_modifiers_except(flags: ModifierFlags) -> ModifierFlags {
    !flags
}

// Go: transformers/estransforms/utilities.go:10 convertClassDeclarationToClassExpression

// Go: transformers/estransforms/utilities.go:23 createNotNullCondition

// Go: transformers/estransforms/utilities.go:55 superAccessState


// Go: transformers/estransforms/utilities.go:280 createAccessorPropertyBackingField
