//! Port of Effect-TS/tsgo `internal/rules/prefer_schema_over_json.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;




fn prefer_schema_over_json_recommendation(is_v4: bool) -> &'static str {
    if is_v4 {
        return "Use `Schema.UnknownFromJsonString` for unknown shapes, `Schema.fromJsonString(schema)` for known ones, or `Schema.toCodecJson(schema)` when working with JSON values instead of strings.";
    }

    "Use `Schema.parseJson(Schema.Unknown)` for unknown shapes or `Schema.parseJson(schema)` for known ones."
}




