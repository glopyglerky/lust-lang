// expect: L0110
// rustc E0080: u128::MAX + 1 overflows in a const argument.
fn f<const N: u128>() -> u128 { N }
fn main() { println!("{}", f::<{ u128::MAX + 1 }>()); }
