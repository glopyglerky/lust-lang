// expect: L0311
// ARCH 2.8, rustc E0505: `a` moves while a live `&dyn` borrows it.
trait Tr { fn m(&self) -> u8; }
struct A(u8);
impl Tr for A { fn m(&self) -> u8 { self.0 } }
fn take(_a: A) {}
fn main() { let a = A(4); let d: &dyn Tr = &a; take(a); println!("{}", d.m()); }
