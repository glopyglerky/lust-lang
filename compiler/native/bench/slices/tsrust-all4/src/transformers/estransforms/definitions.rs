//! Port of Go `transformers/estransforms/definitions.go`.
//!
//! PORT: Go builds the chained factories as package-level `var`s with
//! `transformers.Chain`. Here each one is a named factory fn that calls
//! `chain` with its components.

use super::contract::{};
use crate::prelude::*;

// Go: transformers/estransforms/definitions.go:9 esDecoratorAndClassFields

// Go: transformers/estransforms/definitions.go:10 NewESNextTransformer

// 2026: no new downlevel syntax
// 2025: only module system syntax (import attributes, json modules), untransformed regex modifiers
// 2024: no new downlevel syntax
// 2023: no new downlevel syntax
// 2022: class static blocks and class fields are handled by newClassFieldsTransformer

// Go: transformers/estransforms/definitions.go:16 NewES2021Transformer

// Go: transformers/estransforms/definitions.go:17 NewES2020Transformer

// Go: transformers/estransforms/definitions.go:18 NewES2019Transformer

// Go: transformers/estransforms/definitions.go:19 NewES2018Transformer

// Go: transformers/estransforms/definitions.go:20 NewES2017Transformer

// Go: transformers/estransforms/definitions.go:21 NewES2016Transformer

// Go: transformers/estransforms/definitions.go:24 GetESTransformer
