// expect: L0110
// rustc E0308: a `Self::V(..)` pattern in `impl<T> G<T>` matches `G<T>`
// only, not a `G<u16>` scrutinee.
enum G<T> {
    V(T),
    N,
}
impl<T> G<T> {
    fn bad(g: G<u16>) -> u8 {
        match g {
            Self::V(_) => 1,
            Self::N => 2,
        }
    }
}
fn main() {
    println!("{}", G::<u8>::bad(G::N));
}
