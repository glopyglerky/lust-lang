// expect: L0703
// ARCH 2.8, rustc E0161: a by-value `self` method cannot run on an unsized dyn value.
trait Tr { fn m(&self) -> u8; fn by(self) -> u8; }
struct S;
impl Tr for S { fn m(&self) -> u8 { 1 } fn by(self) -> u8 { 4 } }
fn main() { let s = S; let d: &dyn Tr = &s; println!("{}", d.by()); }
