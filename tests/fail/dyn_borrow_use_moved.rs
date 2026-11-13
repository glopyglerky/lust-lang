// expect: L0301
// ARCH 2.8, rustc E0382: a `&mut dyn` is not Copy; `b = a` moves it.
trait Acc { fn add(&mut self, x: u32); }
struct S(u32);
impl Acc for S { fn add(&mut self, x: u32) { self.0 += x; } }
fn take(_a: &mut dyn Acc) {}
fn main() { let mut s = S(0); let a: &mut dyn Acc = &mut s; let b = a; take(b); a.add(1); }
