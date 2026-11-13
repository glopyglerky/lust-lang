// An empty BTreeMap or BTreeSet checks no range bounds (std's has no root
// to search); a non-empty one panics in std's btree/search.rs (tf-panicloc).
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Bound;
fn main() {
    let n = std::env::args().count();
    let m: BTreeMap<usize, u8> = BTreeMap::new();
    let s: BTreeSet<usize> = BTreeSet::new();
    println!("{} {}", m.range(n + 5..n).count(), s.range(n + 5..n).count());
    println!("{}", m.range((Bound::Excluded(n), Bound::Excluded(n))).count());
    let mut m = m;
    m.insert(1, 2);
    println!("{}", m.range(n + 5..n).count());
}
