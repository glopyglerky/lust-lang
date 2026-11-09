// expect: L0110
// rustc E0384: lust's collect-time scalar evaluator takes no assignment to a binding without `mut`.
enum E {
    A = {
        let a = 1;
        a = 2;
        a
    },
    B,
}
fn main() {
    println!("{}", E::A as isize + E::B as isize);
}
