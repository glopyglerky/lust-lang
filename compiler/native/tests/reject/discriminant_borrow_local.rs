// expect: L0110
// Valid Rust. lust's collect-time scalar evaluator has no borrow checker for
// the anonymous const of a discriminant, so it takes no borrow of a local
// there (a const fn body or a const item may borrow; both are checked).
enum E {
    A = {
        let a = 3;
        let r = &a;
        *r
    },
    B,
}
fn main() {
    println!("{}", E::A as isize + E::B as isize);
}
