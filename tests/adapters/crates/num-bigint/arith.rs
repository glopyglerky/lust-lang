// lust-reject: L0113
// Arithmetic operators are not in the adapter, so `+` on BigInt rejects;
// the real crate adds.
use num_bigint::BigInt;

fn main() {
    let a = BigInt::from(2i64);
    let b = BigInt::from(3i64);
    println!("{}", a + b);
}
