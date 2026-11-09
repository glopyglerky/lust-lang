// expect: L0110
// ARCH 2.8, rustc E0308: upcasting only reaches supertraits; B is unrelated to A.
trait A { fn a(&self) -> u8; }
trait B { fn b(&self) -> u8; }
struct S;
impl A for S { fn a(&self) -> u8 { 1 } }
impl B for S { fn b(&self) -> u8 { 2 } }
fn main() { let s = S; let d: &dyn A = &s; let _e: &dyn B = d; }
