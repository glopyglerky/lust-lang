// expect: L0701
// ARCH 2.8, rustc E0038: a trait that requires `Self: Sized`.
trait T: Sized { fn v(&self) -> u8; }
fn f(_x: &dyn T) {}
fn main() {}
