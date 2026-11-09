// expect: L0110
// rustc E0747: a type provided when a constant was expected (trait input).
trait Tr<const N: usize> { fn v(&self) -> usize; }
fn g<T: Tr<u8>>(_x: T) {}
fn main() {}
