// expect: L0110
// rustc: literal out of range for i32 in a discriminant.
#[repr(i32)]
#[allow(dead_code)]
enum E { A = 340282366920938463463374607431768211455, B }
fn main() { println!("{}", E::B as i32); }
