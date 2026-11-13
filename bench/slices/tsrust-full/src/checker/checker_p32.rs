//! Port of typescript-go `internal/checker/checker.go` lines 28753-29671.
//! Read `crates/ts_goport/PORTING.md` before editing.

use crate::jsnum::Number;
use crate::prelude::*;
use std::borrow::Cow;

// Go: checker/checker.go:29571 maxTemplateLiteralTypeLength, maxTemplateLiteralTypeSpans
// Limits on the size of a template literal type produced by getTemplateLiteralType. Recursive instantiations
// such as `Recur<any, `${S}_${S}`>` double the text (or the number of placeholders) on every iteration and
// exhaust memory long before the tail recursion limit in getConditionalType is reached (see #63271).
const MAX_TEMPLATE_LITERAL_TYPE_LENGTH: usize = 50_000_000;
const MAX_TEMPLATE_LITERAL_TYPE_SPANS: usize = 100_000;




