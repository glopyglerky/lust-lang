// expect: L0701
// ARCH 2.8, rustc E0038: `Self` inside a generic argument of a parameter.
trait Bad { fn f(&self, x: Option<&Self>); }
struct S;
impl Bad for S { fn f(&self, _x: Option<&Self>) {} }
fn main() { let s = S; let _d: &dyn Bad = &s; }
