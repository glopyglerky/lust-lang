// expect: L0110
// ARCH 2.8, rustc E0308: a coercion cannot add an auto trait the source dyn type does not have.
trait A { fn a(&self) -> u8; }
struct S;
impl A for S { fn a(&self) -> u8 { 1 } }
fn main() { let s = S; let d: &dyn A = &s; let _e: &(dyn A + Send) = d; }
