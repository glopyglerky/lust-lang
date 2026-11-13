// expect: L0704
// ARCH 2.8, rustc E0371: `dyn A` implements A already.
trait A { fn a(&self) -> u8; }
impl A for dyn A { fn a(&self) -> u8 { 1 } }
fn main() {}
