// expect: L0110
// ARCH 2.8, rustc E0277: a slice element must be sized.
trait Tr { fn m(&self) -> u8; }
fn f(_x: &[dyn Tr]) {}
fn main() {}
