// expect: L0110
// rustc E0370: the implicit discriminant past i128::MAX overflows.
#[repr(i128)]
#[allow(dead_code)]
enum E { X = i128::MAX, Y }
fn main() { println!("{}", E::Y as i128); }
