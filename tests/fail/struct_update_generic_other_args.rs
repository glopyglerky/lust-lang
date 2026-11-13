// expect: L0110
// rustc E0308: the base of `..` must have the literal's own type
// (type-changing struct update is unstable).
#[derive(Debug)]
struct P<T> {
    x: T,
    y: T,
}

fn main() {
    let a = P { x: 1u8, y: 2u8 };
    let b: P<u16> = P { x: 1, ..a };
    println!("{:?}", b);
}
