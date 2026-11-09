// expect: L0110
// ARCH 2.8, rustc E0746: a return type of `dyn Tr` has no size.
trait Tr { fn m(&self) -> u8; }
fn f(x: &dyn Tr) -> dyn Tr { *x }
fn main() {}
