// expect: L0110
// rustc E0277: only the last field of a struct may be unsized.
struct W<T: ?Sized>(T, u8);
fn main() {}
