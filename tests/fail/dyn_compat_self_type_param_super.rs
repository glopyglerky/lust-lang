// expect: L0701
// ARCH 2.8, rustc rejects: a supertrait that uses `Self` as a type parameter.
trait Pe<R> { fn p(&self, r: &R) -> bool; }
trait T: Pe<Self> { fn v(&self) -> u8; }
fn f(_x: &dyn T) {}
fn main() {}
