// expect: L0110
// rustc E0107: `T` has no default, so `S` alone is missing an argument.
struct S<T, U = u8> {
    a: T,
    b: U,
}

fn main() {
    let s: S = S { a: 1, b: 2 };
    println!("{}", s.a);
}
