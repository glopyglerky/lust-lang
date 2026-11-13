// expect: L0110
// rustc E0308: `Self::mk` in `impl S<u8>` builds an `S<u8>`, not an `S<u16>`.
struct S<T>(T);
impl<T> S<T> {
    fn mk(x: T) -> S<T> {
        S(x)
    }
}
impl S<u8> {
    fn g() -> S<u16> {
        Self::mk(7)
    }
}
fn main() {
    println!("{}", S::<u8>::g().0);
}
