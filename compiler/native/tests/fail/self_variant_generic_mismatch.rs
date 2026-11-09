// expect: L0110
// rustc E0308: `Self::V` in `impl<T> G<T>` takes the impl's `T`, not a
// fresh type argument (lust check accepted this before).
struct D(&'static str);
enum G<T> {
    V(T),
    N,
}
impl<T> G<T> {
    fn go() {
        let _x = Self::V(&D("t"));
        println!("in");
    }
}
fn main() {
    G::<u8>::go();
}
