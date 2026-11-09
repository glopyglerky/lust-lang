// expect: L0110
// rustc E0308: `(&mut s).deref()` is core's `impl Deref for &mut T`
// (`&self` takes no `&mut String` step by value), so it gives `&String`.
use std::ops::Deref;
fn only_str(_: &&str) {}
fn main() {
    let mut s = String::from("s");
    let m = &mut s;
    let o = m.deref();
    only_str(&o);
}
