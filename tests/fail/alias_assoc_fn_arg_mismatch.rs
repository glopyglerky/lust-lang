// expect: L0110
// rustc E0308: through `type A = S<u8>`, `make` takes a u8.
struct S<T>(T);
impl<T> S<T> {
    fn make(t: T) -> Self {
        S(t)
    }
}
type A = S<u8>;
fn main() {
    let a = A::make(300u16);
    println!("{}", a.0);
}
