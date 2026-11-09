// expect: L0720
// ARCH 2.8, rustc E0034: an `impl dyn` method and a trait method with one name.
trait Tr { fn m(&self) -> u8; }
struct S;
impl Tr for S { fn m(&self) -> u8 { 1 } }
impl dyn Tr { fn m(&self) -> u8 { 2 } }
fn main() { let s = S; let d: &dyn Tr = &s; println!("{}", d.m()); }
