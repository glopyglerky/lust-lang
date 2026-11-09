// expect: L0309
// rustc E0594: `BTreeMap` has no `IndexMut`, so assigning
// through `map[&k]` writes through the shared `Index` reference.
use std::collections::BTreeMap;
fn main() {
    let mut m = BTreeMap::from([(1, 10)]);
    m[&1] = 20;
    println!("{}", m[&1]);
}
