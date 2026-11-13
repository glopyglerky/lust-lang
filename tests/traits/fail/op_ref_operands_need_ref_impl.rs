// rustc-error: E0369
// lust-expect: L0113
// ARCH 2.5: operator impls apply to the operand types as written; impl Add
// for V gives nothing for &V + &V (core's reference impls are scalar only).
use std::ops::Add;
#[derive(Debug, Clone, Copy)]
struct V(i32);
impl Add for V { type Output = V; fn add(self, o: V) -> V { V(self.0 + o.0) } }
fn main() { let x = &V(1) + &V(2); println!("{:?}", x); }
