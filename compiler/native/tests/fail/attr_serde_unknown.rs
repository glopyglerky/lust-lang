// expect: L0402
// rustc error here: cannot find attribute `serde` in this scope (no
// serde derive in this file). With the derive, lust rejects the
// derive itself, so the helper attribute never gets a silent accept.
struct S {
    #[serde(default)]
    x: i64,
}
fn main() {
    let s = S { x: 1 };
    println!("{}", s.x);
}
