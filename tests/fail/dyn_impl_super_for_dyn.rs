// expect: L0704
// ARCH 2.8, rustc E0371: `dyn B` implements its supertrait A already.
trait A { fn a(&self) -> u8; }
trait B: A { fn b(&self) -> u8; }
impl A for dyn B { fn a(&self) -> u8 { 1 } }
fn main() {}
