// expect: L0113
// rustc E0308: impl AddAssign for V takes a V, not &V.
use std::ops::AddAssign;
#[derive(Debug, Clone, Copy)]
struct V(i32);
impl AddAssign for V { fn add_assign(&mut self, o: V) { self.0 += o.0; } }
fn main() { let mut a = V(1); a += &V(2); println!("{:?}", a); }
