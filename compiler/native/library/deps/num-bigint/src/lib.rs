// Lust adapter for num-bigint 0.4.6 (docs/ARCH.md section 3.6): the
// `BigInt` subset ts-rust uses (jsnum.rs: parse_bytes, From<i64>, pow,
// to_f64 through num_traits::ToPrimitive, Display), plus cheap exact
// extras. Values are exact; digits are little-endian u64 limbs, as
// upstream's on 64-bit targets. Not here (they reject at the use):
// BigUint, arithmetic operators, Num/Zero/One/Signed/Pow impls,
// FromStr, Hash, to_f32, and the bit and radix-digit APIs.

mod bigint;

pub use crate::bigint::{BigInt, Sign};
