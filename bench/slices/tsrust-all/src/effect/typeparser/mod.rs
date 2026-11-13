//! Port of Effect-TS/tsgo `internal/typeparser` at `@effect/tsgo@0.46.1`
//! (`f1a7cad0`). It recognizes Effect values, services, layers, schemas and
//! call shapes from checker types. One file per Go file, in Go order.
//!
//! Go `Cached(&tp.links.X, key, compute)` is `cached!(self, x, key, compute)`.
//! The links live on the checker (Go `Checker.EffectLinks`), so every
//! `TypeParser` of one checker shares them.

use crate::prelude::*;

mod cause_type;
mod constant_evaluation;
mod context_tag;
mod context_type;
mod could_be_strict_effect;
mod data_first_signature;
mod data_type;
mod discover;
mod effect_context;
mod effect_fn;
mod effect_fn_opportunity;
mod effect_gen;
mod effect_model_type;
mod effect_type;
mod effect_yieldable_type;
mod execution_flow;
mod expected_and_real_type;
mod expression_stability;
mod extends_context_service;
mod extends_context_tag;
mod extends_data_tagged;
mod extends_effect_model_class;
mod extends_effect_service;
mod extends_effect_tag;
mod extends_schema_class;
mod extends_schema_opaque;
mod extends_schema_tagged;
mod extends_sql_model_class;
mod function_node;
mod get_type_at_location;
mod global_error;
mod helpers;
mod identity_forwarder;
mod layer_type;
mod lazy_expression;
mod module_export_reference;
mod module_identifier;
mod object_literal;
mod option_type;
mod packagejson;
mod pipeable;
mod pipeable_signature_shape;
mod piping_flow;
mod promise_type;
mod reconstruct;
mod result_dispatch;
mod returning_dispatch;
mod schema_type;
mod scope_type;
mod service_type;
mod sql_model_type;
mod stream_type;
mod unique_types_map;
mod unroll_members;
mod vitest_api;
mod yieldable_error;

pub use cause_type::*;
pub use constant_evaluation::*;
pub use context_tag::*;
pub use context_type::*;
pub use could_be_strict_effect::*;
pub use data_first_signature::*;
pub use data_type::*;
pub use discover::*;
pub use effect_context::*;
pub use effect_fn::*;
pub use effect_fn_opportunity::*;
pub use effect_gen::*;
pub use effect_model_type::*;
pub use effect_type::*;
pub use effect_yieldable_type::*;
pub use execution_flow::*;
pub use expected_and_real_type::*;
pub use expression_stability::*;
pub use extends_context_service::*;
pub use extends_context_tag::*;
pub use extends_data_tagged::*;
pub use extends_effect_model_class::*;
pub use extends_effect_service::*;
pub use extends_effect_tag::*;
pub use extends_schema_class::*;
pub use extends_schema_opaque::*;
pub use extends_schema_tagged::*;
pub use extends_sql_model_class::*;
pub use function_node::*;
pub use get_type_at_location::*;
pub use global_error::*;
pub use helpers::*;
pub use identity_forwarder::*;
pub use layer_type::*;
pub use lazy_expression::*;
pub use module_export_reference::*;
pub use module_identifier::*;
pub use object_literal::*;
pub use option_type::*;
pub use packagejson::*;
pub use pipeable::*;
pub use pipeable_signature_shape::*;
pub use piping_flow::*;
pub use promise_type::*;
pub use reconstruct::*;
pub use result_dispatch::*;
pub use returning_dispatch::*;
pub use schema_type::*;
pub use scope_type::*;
pub use service_type::*;
pub use sql_model_type::*;
pub use stream_type::*;
pub use unique_types_map::*;
pub use unroll_members::*;
pub use vitest_api::*;
pub use yieldable_error::*;



/// Go `Cached(&tp.links.<field>, key, compute)`: the cached value, or
/// `compute` stored and returned. `compute` may use the type parser.
#[macro_export]
#[doc(hidden)]
macro_rules! effect_cached {
    ($tp:expr, $field:ident, $key:expr, $compute:expr) => {{
        let key = $key;
        if let Some(value) = $tp.links().$field.get(&key) {
            value.clone()
        } else {
            let value = $compute;
            $tp.links().$field.insert(key, value.clone());
            value
        }
    }};
}
pub use crate::effect_cached as cached;

