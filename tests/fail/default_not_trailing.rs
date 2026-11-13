// expect: L0110
// rustc: generic parameters with a default must be trailing.
struct S<T = i32, U> {
    a: T,
    b: U,
}

fn main() {}
