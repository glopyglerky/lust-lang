// expect: L0113
// rustc E0600: impl Neg for V gives no Neg for &V.
use std::ops::Neg;
#[derive(Debug, Clone, Copy)]
struct V(i32);
impl Neg for V { type Output = V; fn neg(self) -> V { V(-self.0) } }
fn main() { let x = -&V(1); println!("{:?}", x); }
