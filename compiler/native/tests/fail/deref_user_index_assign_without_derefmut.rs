// expect: L0308
// rustc E0594: assigning through an index on a user `Deref` needs `DerefMut`.
use std::ops::Deref;
struct Sv { items: Vec<u32> }
impl Deref for Sv { type Target = [u32]; fn deref(&self) -> &[u32] { &self.items } }
fn main() { let mut s = Sv { items: vec![1] }; s[0] = 5; }
