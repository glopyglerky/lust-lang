// BTreeMap/BTreeSet range and range_mut (W-STD-C phase 2): all
// bound kinds, range expressions and Bound pairs (owned and by
// reference), borrowed keys, reverse iteration, mutation.
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Bound::{Excluded, Included, Unbounded};
fn main() {
    let m = BTreeMap::from([(1, 10), (2, 20), (3, 30), (5, 50), (8, 80)]);
    let v: Vec<i32> = m.range(2..5).map(|(k, _)| *k).collect();
    let w: Vec<i32> = m.range((Excluded(2), Included(5))).map(|(k, _)| *k).collect();
    let x: Vec<i32> = m.range((Included(&3), Unbounded)).map(|(k, _)| *k).collect();
    let y: Vec<i32> = m.range(..).rev().map(|(k, _)| *k).collect();
    println!("{v:?} {w:?} {x:?} {y:?}");
    let mut m = m;
    for (k, val) in m.range_mut(2..=5) {
        *val += *k;
    }
    println!("{m:?}");
    let s = BTreeSet::from([1, 2, 3, 5, 8]);
    let z: Vec<i32> = s.range((Excluded(&1), Included(&5))).copied().collect();
    println!("{z:?}");
    let sm: BTreeMap<String, i32> = vec![("a".to_string(), 1), ("cc".to_string(), 2), ("e".to_string(), 3)].into_iter().collect();
    let ks: Vec<&String> = sm.range((Included(String::from("b")), Excluded(String::from("e")))).map(|(k, _)| k).collect();
    println!("{ks:?}");
}
