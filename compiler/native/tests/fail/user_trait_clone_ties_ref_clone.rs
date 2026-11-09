// expect: L0720
// rustc E0034: `Clone for &S` and the user's `fn clone(&self)` for `&S`
// both take the `&&S` receiver by value at the first step.
#![allow(noop_method_call)]
#[derive(Clone)]
struct S;
trait Sh {
    fn clone(&self) -> u8;
}
impl Sh for &S {
    fn clone(&self) -> u8 {
        1
    }
}
fn main() {
    let s = S;
    let r = &s;
    let rr = &r;
    let _c = rr.clone();
}
