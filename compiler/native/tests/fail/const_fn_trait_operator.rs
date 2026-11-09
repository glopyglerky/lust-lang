// expect: L0631
// rustc E0015: an operator through a trait impl is a non-const call.
use std::ops::Add;
struct S(i32);
impl Add for S { type Output = S; fn add(self, r: S) -> S { S(self.0 + r.0) } }
const fn h(a: S, b: S) -> S { a + b }
fn main() { println!("{}", h(S(1), S(2)).0); }
