// panic: no entry found for key
// `map[&k]` for a missing key panics with std's text
// (library/alloc/src/collections.rs, W-STD-C phase 2).
use std::collections::BTreeMap;
fn main() {
    let m = BTreeMap::from([(1, 10), (2, 20)]);
    println!("{}", m[&2]);
    println!("{}", m[&3]);
}
