// expect: L0110
// ARCH 2.8, rustc E0308: `&S` cannot become `&mut dyn A`.
trait A { fn a(&self) -> u8; }
struct S;
impl A for S { fn a(&self) -> u8 { 1 } }
fn main() { let s = S; let _d: &mut dyn A = &s; }
