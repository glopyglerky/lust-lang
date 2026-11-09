// expect: L0001
// rustc: expected `]`, found `x`.
fn main() {
    #[allow(unused) x]
    let a = 1;
}
