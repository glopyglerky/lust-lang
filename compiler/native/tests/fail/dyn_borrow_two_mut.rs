// expect: L0304
// ARCH 2.8, rustc E0499: two `&mut dyn` borrows of one value at once.
trait Tr { fn bump(&mut self); }
struct S { n: u8 }
impl Tr for S { fn bump(&mut self) { self.n += 1; } }
fn main() { let mut s = S { n: 0 }; let d: &mut dyn Tr = &mut s; let e: &mut dyn Tr = &mut s; d.bump(); e.bump(); }
