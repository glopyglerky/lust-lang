// expect: L0110
// rustc E0308: the pattern's turbofish names `E<u16>`, the scrutinee is `E<u8>`.
enum E<T> {
    A(T),
    B,
}

fn main() {
    let e = E::<u8>::A(1);
    match e {
        E::<u16>::A(v) => println!("{}", v),
        E::B => {}
    }
}
