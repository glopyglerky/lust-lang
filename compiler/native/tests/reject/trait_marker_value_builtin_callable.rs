// expect: L0641
// Valid Rust (prints `[1, 2, 3]`): `Clone::clone` at `i32` has no fn in
// lust, so a builtin that calls it through a C adapter rejects.
fn main() {
    let mut v = vec![3, 1, 2];
    v.sort_by_key(Clone::clone);
    println!("{:?}", v);
}
