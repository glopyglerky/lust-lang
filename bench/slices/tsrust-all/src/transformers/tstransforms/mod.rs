//! Go package `transformers/tstransforms`: the TypeScript-only script
//! transforms (type erasure, import elision, runtime syntax, legacy
//! decorators and decorator metadata).
//!
//! PORT: Go embeds `transformers.Transformer`, which owns one root visitor
//! (`tx.Visitor()`) whose `Visit` is `tx.visit`. Here each transformer
//! implements `TransformerVisit`, and [`TxVisit`] adds the Go
//! `tx.Visitor().X(...)` helpers on top of it. Go
//! `tx.Visitor().VisitNode(n)` is `self.visit_node(n)`.

use crate::prelude::*;

pub mod import_elision;
pub mod legacy_decorators;
pub mod metadata;
pub mod runtime_syntax;
pub mod type_eraser;
pub mod type_serializer;
pub mod utilities;

pub use import_elision::{};
pub use legacy_decorators::{};
pub use metadata::{};
pub use runtime_syntax::{};
pub use type_eraser::{};


