// expect: L0202
// Sol review probe `impl_omitted_lifetime`; rustc: E0726 implicit elided lifetime not allowed here
struct S<'a>(&'a i32);
trait Tag {}
impl Tag for S {}
fn main() {}
