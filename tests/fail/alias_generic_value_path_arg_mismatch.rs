// expect: L0110
// rustc E0308: `A::<u8>::make` takes a u8.
struct S<T>(T);
impl<T> S<T> {
    fn make(t: T) -> Self {
        S(t)
    }
}
type A<T> = S<T>;
fn main() {
    let a = A::<u8>::make(300u16);
    println!("{}", a.0);
}
