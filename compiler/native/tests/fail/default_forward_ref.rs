// expect: L0110
// rustc E0128: generic parameter defaults cannot reference parameters before they are declared.
struct S<T = U, U = i32> {
    a: T,
    b: U,
}

fn main() {}
