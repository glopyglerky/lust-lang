// expect: L0641
// Valid Rust (prints `1`): `<i32 as Clone>::clone` has no fn in lust (a
// builtin impl), so a fn pointer to it is a reject.
fn main() {
    let f = <i32 as Clone>::clone;
    let g: fn(&i32) -> i32 = f;
    println!("{}", g(&1));
}
