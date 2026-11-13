# Adapter: ryu-js 1.0.2

Lust adapter crate (docs/ARCH.md section 3.6). Dev builds compile it in
place of the registry package ryu-js 1.0.2; release builds use the real
crate through Cargo. Gate: `scripts/test-adapters.sh ryu-js` (the real
crate is the oracle, on random f64 bits).

## Implemented, same results as upstream

- `Buffer::new`, `Buffer::format` for f64 (ECMAScript Number::toString:
  `NaN`, `Infinity`, `-Infinity`, `0` for -0.0), `Clone` (a fresh
  buffer, as upstream), `Default`, and the `Float` trait for f64. The
  text comes from the runtime's JS printer (`lust_fmt_js_f64`,
  src/runtime/float.c) through the `ryu_js_fmt_f64` intrinsic.
- Feature: `small` (upstream's smaller tables; same digits).

## Rejected (lust error, never a different answer)

- f32 (`Float` has no f32 impl: L0612), `format_finite`,
  `format_to_fixed`, `FloatToFixed`, and `raw` (L0104 or L0506).
- `size_of` and other layout queries on `Buffer` (L0652).
