// expect: L0113
// rustc E0308: impl Add for V takes a V on the right, not &V.
use std::ops::Add;
#[derive(Debug, Clone, Copy)]
struct V(i32);
impl Add for V { type Output = V; fn add(self, o: V) -> V { V(self.0 + o.0) } }
fn main() { let x = V(1) + &V(2); println!("{:?}", x); }
