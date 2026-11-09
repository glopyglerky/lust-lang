// expect: L0118
// rustc: the turbofish fixes `T = u8`, so 300 is out of range.
struct S<T> {
    a: T,
}

fn main() {
    let t = S::<u8> { a: 300 };
    println!("{}", t.a);
}
