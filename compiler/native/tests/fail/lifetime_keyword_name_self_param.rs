// expect: L0001
// rustc: lifetimes cannot use keyword names, in a `&'a self` receiver too.
struct S;
impl S { fn f(&'gen self) {} }
fn main() { S.f(); }
