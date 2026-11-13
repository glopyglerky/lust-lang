// rustc-error: E0119
// lust-expect: L0615
// ARCH 2.6: two impls with the same Self and the same trait
// inputs conflict. Contrast with pass/same_self_diff_inputs.rs,
// where the inputs differ and both impls stand.
struct S;
trait Tr<A> {}
impl Tr<u8> for S {}
impl Tr<u8> for S {}
fn main() {}
