// expect: L0104
// rustc E0599: autoderef through a user Deref steps to the target place S, not to &S,
// so the impl for &S (receiver &&S) is never reached.
use std::ops::Deref;
struct S;
struct W(S);
impl Deref for W { type Target = S; fn deref(&self) -> &S { &self.0 } }
trait Tr { fn m(&self) -> &'static str; }
impl Tr for &S { fn m(&self) -> &'static str { "trait for &S" } }
fn main() {
    let w = W(S);
    println!("{}", w.m());
}
