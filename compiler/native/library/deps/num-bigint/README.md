# Adapter: num-bigint 0.4.6

Lust adapter crate (docs/ARCH.md section 3.6). Dev builds compile it in
place of the registry package num-bigint 0.4.6; release builds use the
real crate through Cargo. Gate: `scripts/test-adapters.sh num-bigint`.

## Implemented, same results as upstream

- `BigInt::parse_bytes` (signs, `_` separators, radix 2 to 36, UTF-8
  check first), `pow`, `sign`, `bits`, `to_str_radix`.
- `From<i64>`, `From<u64>`, `From<i32>`, `From<u32>`, `Clone`,
  `Default`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Display` (through
  `pad_integral`, so format flags apply) and `Debug` (as Display).
- `num_traits::ToPrimitive`: `to_i64`, `to_u64`, `to_i128`, `to_u128`,
  `to_f64` (upstream's round-to-odd top bits), and the provided ones.
- `Sign` with its derives (no `Hash`).
- Features: `default`, `std`.

Limbs are little-endian u64, as upstream's on 64-bit targets.

## Rejected (lust error, never a different answer)

- `BigUint`, arithmetic and bit operators (L0113), `Hash`, `FromStr`,
  `Num`/`Zero`/`One`/`Signed`/`Pow` impls, `to_f32`, the other `From`
  impls, `TryFrom`, and the byte and digit APIs.
- Layout queries on `BigInt` (L0652).
