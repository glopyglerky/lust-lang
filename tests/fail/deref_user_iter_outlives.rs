// expect: L0306
// rustc E0597: an iterator from a user `Deref` target borrows the receiver.
use std::ops::Deref;
struct G { items: Vec<u32> }
impl Deref for G { type Target = [u32]; fn deref(&self) -> &[u32] { &self.items } }
fn main() { let it; { let g = G { items: vec![3, 1] }; it = g.iter(); } println!("{:?}", it.count()); }
