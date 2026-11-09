// expect: L0110
// rustc E0308: the field of `K<u8>` is `Option<u8>`.
struct K<T> {
    x: Option<T>,
    n: u8,
}
static S1: K<u8> = K { x: Some(5u16), n: 1 };
fn main() {
    println!("{}", S1.n);
}
