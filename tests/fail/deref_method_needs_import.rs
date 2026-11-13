// expect: L0104
// rustc E0599: `Deref` is not in the prelude, so `s.deref()` needs `use std::ops::Deref`.
fn main() {
    let s = String::from("a");
    let r = s.deref();
    println!("{}", r);
}
