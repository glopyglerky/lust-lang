// expect: L0110
// rustc: generic parameters with a default must be trailing (a const one too).
struct S<const N: usize = 3, T> {
    a: [T; N],
}

fn main() {}
