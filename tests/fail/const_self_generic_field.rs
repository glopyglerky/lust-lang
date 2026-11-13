// expect: L0110
// rustc: generic `Self` types are currently not permitted in anonymous
// constants. `Self` in a field length of a generic struct is `G<T>`.
struct G<T> {
    v: Vec<T>,
    s: [u8; Self::N],
}
impl<T> G<T> {
    const N: usize = 4;
}
fn main() {}
