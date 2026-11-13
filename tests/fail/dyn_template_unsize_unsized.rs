// expect: L0110
// ARCH 2.8, rustc E0277: a `?Sized` param may be dyn already, so the template cannot unsize it.
trait Tr { fn m(&self) -> u8; }
fn h<T: Tr + ?Sized>(x: &T) -> &dyn Tr { x }
fn main() {}
