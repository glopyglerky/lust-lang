// expect: L0110
// rustc: constant expression depends on a generic parameter. `Self::D` is
// `N` here, so the repeat length depends on the impl's const parameter.
struct S<const N: usize>;
impl<const N: usize> S<N> {
    const D: usize = N;
    fn m(&self) -> usize {
        let a = [0u8; Self::D];
        a.len()
    }
}
fn main() {
    println!("{}", S::<4>.m());
}
