// expect: L0110
// rustc E0596: `&mut w` to `&mut P` needs `DerefMut`.
use std::ops::Deref;
struct P { x: i32 }
struct W { p: P }
impl Deref for W { type Target = P; fn deref(&self) -> &P { &self.p } }
fn bump(p: &mut P) { p.x += 1; }
fn main() { let mut w = W { p: P { x: 1 } }; bump(&mut w); }
