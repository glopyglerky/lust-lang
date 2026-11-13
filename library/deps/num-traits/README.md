# Adapter: num-traits 0.2.19

Lust adapter crate (docs/ARCH.md section 3.6). Dev builds compile it in
place of the registry package num-traits 0.2.19; release builds use the
real crate through Cargo. Gate: `scripts/test-adapters.sh num-traits`.

## Implemented, same results as upstream

- `ToPrimitive` (also as `cast::ToPrimitive`): `to_i8` through
  `to_i128`, `to_isize`, `to_u8` through `to_u128`, `to_usize`, `to_f64`,
  with upstream's provided-method answers.
- `impl ToPrimitive for f64` with upstream's exact range rules (the
  num-bigint adapter adds `BigInt`).
- Features: `default`, `i128`, `std`.

## Rejected (lust error, never a different answer)

- `to_f32` (L0104): its u64-to-f32 rounding is not proven exact on
  lust's C backend yet.
- `ToPrimitive` for the integer types and f32, and every other trait
  and fn (Num, Zero, One, Pow, Signed, Float, NumCast, FromPrimitive,
  AsPrimitive, cast, ...): L0104, L0506, or L0612 at the use.
- Feature `libm` (L0508).
