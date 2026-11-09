// expect: L0113
// rustc E0369: impl Add for V gives no Add for &V; operator impls match the operand types as written.
use std::ops::Add;
#[derive(Debug, Clone, Copy)]
struct V(i32);
impl Add for V { type Output = V; fn add(self, o: V) -> V { V(self.0 + o.0) } }
fn main() { let x = &V(1) + &V(2); println!("{:?}", x); }
