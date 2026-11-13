// expect: L0110
// rustc: literal out of range for i128 in a const argument.
fn g<const V: i128>() -> i128 { V }
fn main() { println!("{}", g::<{ 170141183460469231731687303715884105728 / 3 }>()); }
