// expect: L0110
// rustc E0107: three arguments for two parameters.
struct S<T, U = u8> {
    a: T,
    b: U,
}

fn main() {
    let s: S<i32, u8, u8> = S { a: 1, b: 2 };
    println!("{}", s.a);
}
