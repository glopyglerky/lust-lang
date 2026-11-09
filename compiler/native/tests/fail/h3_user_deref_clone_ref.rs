// expect: L0104
// rustc E0599: W is not Clone and S is not Clone; the &S that deref() returns is no
// probe step, so `<&S as Clone>::clone` does not apply.
use std::ops::Deref;
struct S(i32);
struct W(S);
impl Deref for W { type Target = S; fn deref(&self) -> &S { &self.0 } }
fn main() {
    let w = W(S(4));
    let c = w.clone();
    println!("{}", c.0);
}
