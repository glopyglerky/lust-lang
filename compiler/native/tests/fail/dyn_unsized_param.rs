// expect: L0110
// ARCH 2.8, rustc E0277: a parameter of type `dyn Tr` has no size.
trait Tr { fn m(&self) -> u8; }
fn f(x: dyn Tr) -> u8 { x.m() }
fn main() {}
