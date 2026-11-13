// expect: L0306
// ARCH 2.8, rustc E0597: the `&dyn` outlives the value it points at.
trait Tr { fn m(&self) -> u8; }
struct A(u8);
impl Tr for A { fn m(&self) -> u8 { self.0 } }
fn main() { let d: &dyn Tr; { let a = A(4); d = &a; } println!("{}", d.m()); }
