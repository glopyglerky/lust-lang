// expect: L0110
// rustc E0109: type arguments are not allowed on a path through `Self`.
enum G<T> {
    V(T),
    N,
}
impl<T> G<T> {
    fn mk(t: T) -> Self {
        Self::V::<T>(t)
    }
}
fn main() {
    let _ = G::mk(1u8);
}
