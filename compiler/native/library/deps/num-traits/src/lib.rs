// Lust adapter for num-traits 0.2.19 (docs/ARCH.md section 3.6): the
// `ToPrimitive` subset that ts-rust uses (jsnum.rs), with impls for f64
// here and for BigInt in the num-bigint adapter. Every other trait (Num,
// Zero, One, Pow, Signed, NumCast, FromPrimitive, ...) is not here and
// rejects at the use.

pub mod cast;

pub use crate::cast::ToPrimitive;
