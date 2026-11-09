// expect: L0612
// ARCH 2.8, rustc E0277: a `?Sized` param cannot pass where `Sized` is needed.
trait Tr { fn m(&self) -> u8; }
fn g<U: Tr>(y: &U) -> u8 { y.m() }
fn h<T: Tr + ?Sized>(x: &T) -> u8 { g(x) }
fn main() {}
