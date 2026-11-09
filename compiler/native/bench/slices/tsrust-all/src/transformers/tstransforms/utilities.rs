//! Port of `transformers/tstransforms/utilities.go`.

use crate::prelude::*;

// Go: transformers/tstransforms/utilities.go:9 constantExpression
// PORT: Go takes `any`; the values used are `string` and `jsnum.Number`,
// which are `LiteralValue::String` and `LiteralValue::Number`. Any other
// value is Go `nil`.

