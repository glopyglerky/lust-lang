// expect: L0308
// rustc E0596: `DerefMut` through a shared reference to the receiver.
use std::ops::{Deref, DerefMut};
struct N { v: u32 }
impl N { fn bump(&mut self) { self.v += 1; } }
struct W { n: N }
impl Deref for W { type Target = N; fn deref(&self) -> &N { &self.n } }
impl DerefMut for W { fn deref_mut(&mut self) -> &mut N { &mut self.n } }
fn f(w: &W) { w.bump(); }
fn main() { let w = W { n: N { v: 1 } }; f(&w); }
