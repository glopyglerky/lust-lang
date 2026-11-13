// expect: L0701
// ARCH 2.8, rustc E0038: a supertrait that is not dyn compatible.
trait Base { fn make() -> Self; }
trait Sub: Base { fn v(&self) -> u8; }
fn f(_x: &dyn Sub) {}
fn main() {}
