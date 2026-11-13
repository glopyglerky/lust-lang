//! Port of Effect-TS/tsgo `internal/rules/schema_sync_in_effect.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

/// syncToEffectMethodV3 maps Schema sync method names to their Effect-based V3 equivalents.
// PORT: Go `map[string]string`; the port keeps the pairs in Go source order.
// Go ranges over the map in random order; every caller matches at most one
// name, so the order does not change a result.
// Go: rules/schema_sync_in_effect.go syncToEffectMethodV3
pub static SYNC_TO_EFFECT_METHOD_V3: &[(&str, &str)] = &[
    ("decodeSync", "decode"),
    ("decodeUnknownSync", "decodeUnknown"),
    ("encodeSync", "encode"),
    ("encodeUnknownSync", "encodeUnknown"),
];

/// syncToEffectMethodV4 maps Schema sync method names to their Effect-based V4 equivalents.
// Go: rules/schema_sync_in_effect.go syncToEffectMethodV4
pub static SYNC_TO_EFFECT_METHOD_V4: &[(&str, &str)] = &[
    ("decodeSync", "decodeEffect"),
    ("decodeUnknownSync", "decodeUnknownEffect"),
    ("encodeSync", "encodeEffect"),
    ("encodeUnknownSync", "encodeUnknownEffect"),
];




