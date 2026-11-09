// `Self::D` in a generic impl whose value uses no generic parameter: rustc
// accepts it (a future-compat warning only); a const parameter length too.
struct S<T>(T);
impl<T> S<T> {
    const D: usize = 3;
    fn m(&self) -> usize {
        let a = [0u8; Self::D];
        a.len()
    }
}
struct C<const N: usize>;
impl<const N: usize> C<N> {
    fn m(&self) -> usize {
        let a = [1u16; N];
        a.len()
    }
}
fn main() {
    println!("{} {}", S(1u8).m(), C::<5>.m());
}
