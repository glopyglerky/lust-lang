// expect: L0110
// rustc E0308: a variant through `type EA = E<u8>` takes `u8`.
enum E<T> {
    A(T),
    C,
}
type EA = E<u8>;
fn main() {
    let e = EA::A(300u16);
    let _ = e;
    let _ = E::<u8>::C;
}
