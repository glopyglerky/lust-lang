// expect: L0110
// rustc E0308: a set of `u8` takes `u8`.
use std::collections::BTreeSet;
fn main() {
    let mut s: BTreeSet<u8> = BTreeSet::new();
    s.insert("x");
}
