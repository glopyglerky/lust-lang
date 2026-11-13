// Lust adapter for ryu-js 1.0.2 (docs/ARCH.md section 3.6): the
// ECMAScript Number::toString of an f64 through `Buffer::format`. The
// text comes from the runtime's JS number printer (lust_fmt_js_f64 in
// src/runtime/float.c, already fuzzed against ryu-js 1.0.2) through the
// `ryu_js_fmt_f64` intrinsic. Not here (they reject at the use): f32,
// `format_finite`, `format_to_fixed` and `FloatToFixed`, and `raw`.

mod buffer;

pub use crate::buffer::{Buffer, Float};
