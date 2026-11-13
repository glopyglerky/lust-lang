// map[&k] through BTreeMap's Index impl (W-STD-C phase 2).
use std::collections::BTreeMap;
fn main() {
    let mut m = BTreeMap::new();
    m.insert(1, "a");
    println!("{}", m[&1]);
}
