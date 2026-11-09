// rustc-error: E0277
// lust-expect: L0110
// ARCH 2.2 derives: every field of a derived Hash must be Hash; f64 is not.
#[derive(Hash)]
struct S(u8, f64);
fn main() {}
