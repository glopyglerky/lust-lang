// expect: L0110
// rustc E0308: the pattern names `S<u16>`, the value is `S<u8>`.
struct S<T> {
    a: T,
}

fn main() {
    let s = S::<u8> { a: 1 };
    let S::<u16> { a } = s;
    println!("{}", a);
}
