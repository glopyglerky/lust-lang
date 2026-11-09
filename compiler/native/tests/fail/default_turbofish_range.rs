// expect: L0118
// rustc: `U` defaults to u8 under the turbofish, so 300 is out of range.
struct S<T, U = u8> {
    a: T,
    b: U,
}

fn main() {
    let t = S::<i32> { a: 1, b: 300 };
    println!("{} {}", t.a, t.b);
}
