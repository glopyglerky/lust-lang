// expect: L0312
// ARCH 2.8, rustc E0506: `s` is borrowed through a live `&dyn`.
trait Tr { fn get(&self) -> u8; }
struct S { n: u8 }
impl Tr for S { fn get(&self) -> u8 { self.n } }
fn main() { let mut s = S { n: 0 }; let d: &dyn Tr = &s; s.n = 3; println!("{}", d.get()); }
