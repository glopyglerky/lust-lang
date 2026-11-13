// expect: L0610
// rustc E0283: `BTreeSet<i32>: Extend<i32>` and `Extend<&i32>` both apply.
use std::collections::BTreeSet;
fn main() {
    let mut s = BTreeSet::from([8, 4, 7]);
    s.extend(vec![]);
    println!("{s:?}");
}
