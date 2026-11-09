struct A<const N: usize>;
impl<const N: usize> A<N> { const M: usize = N + 1; }
trait G { const V: usize; }
struct S<const N: usize>;
impl<const N: usize> G for S<N> { const V: usize = N * 2; }
fn main() { println!("{} {}", A::<3>::M, <S<4> as G>::V); }
