// expect: L0312
// ARCH 2.8, rustc E0506: `s` is borrowed through a live `&mut dyn`.
trait Tr { fn bump(&mut self); }
struct S { n: u8 }
impl Tr for S { fn bump(&mut self) { self.n += 1; } }
fn main() { let mut s = S { n: 0 }; let d: &mut dyn Tr = &mut s; s.n = 5; d.bump(); }
