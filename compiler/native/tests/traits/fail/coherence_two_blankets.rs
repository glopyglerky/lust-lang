// rustc-error: E0119
// lust-expect: L0615
// ARCH 2.6: `impl<T: A> Tr for T` and `impl<T: B> Tr for T`
// overlap even when A, B, and Tr are all local and A and B have
// no impls yet. A bare param Self can never be disproved.
// From the sol probe r2-coherence.rs.
trait A {}
trait B {}
trait Tr {}
impl<T: A> Tr for T {}
impl<T: B> Tr for T {}
fn main() {}
