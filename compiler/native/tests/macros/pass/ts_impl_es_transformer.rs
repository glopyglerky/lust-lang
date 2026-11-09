// macro: impl_es_transformer
// source: crates/ts_goport/src/transformers/estransforms/utilities.rs:106
// The macro_rules! text below is copied exact from ts-rust. Only the
// surrounding types are minimal stand-ins. See tests/macros/README.md.
use std::rc::Rc;

pub type Node = i32;
pub mod printer {
    #[derive(Debug)]
    pub struct EmitContext { pub tag: i32 }
}
pub mod transformers {
    pub mod estransforms {
        pub mod utilities {
            use crate::{EmitContext, Node};
            use std::rc::Rc;
            pub trait TxVisitors {
                fn ec(&self) -> Rc<EmitContext>;
                fn root_visit(&mut self, node: Node) -> Node;
                fn transform_source_file_impl(&mut self, file: Node) -> Node { self.root_visit(file) + 100 }
            }
        }
        pub mod contract {
            use crate::{EmitContext, Node};
            use std::rc::Rc;
            pub trait Transformer {
                fn emit_context(&self) -> &Rc<EmitContext>;
                fn transform_source_file(&mut self, file: Node) -> Node;
            }
        }
    }
}
pub use printer::EmitContext;
use transformers::estransforms::contract::Transformer;
use transformers::estransforms::utilities::TxVisitors;

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

pub struct UsingTx { pub emit_context: Rc<EmitContext> }
impl UsingTx {
    pub fn visit(&mut self, node: Node) -> Node { node + 1 }
}
impl_es_transformer!(UsingTx);

fn main() {
    let mut tx = UsingTx { emit_context: Rc::new(EmitContext { tag: 9 }) };
    println!("ec={} root={} file={} ctx={}", tx.ec().tag, tx.root_visit(1), tx.transform_source_file(1), tx.emit_context().tag);
}
