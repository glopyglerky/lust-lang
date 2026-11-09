// expect: L0110
// Sol review probe `impl_unconstrained_assoc`; rustc: E0207 the lifetime parameter `'a` is not constrained by the impl trait, self type, or predicates
trait Has { type Item; }
struct S;
impl<'a> Has for S { type Item = &'a i32; }
fn main() {}
