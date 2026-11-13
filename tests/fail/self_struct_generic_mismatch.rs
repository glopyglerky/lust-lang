// expect: L0110
// rustc E0308: `Self { .. }` in `impl<T> G<T>` builds `G<T>`.
struct G<T> {
    v: T,
}
impl<T> G<T> {
    fn bad() -> u8 {
        let _x = Self { v: 5u16 };
        1
    }
}
fn main() {
    println!("{}", G::<u8>::bad());
}
