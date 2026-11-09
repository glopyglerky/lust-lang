// `v.extend(i)` in a generic body with I = a std collection iterator struct by value
// (`btree_map::Values`, `vec_deque::Iter`, ...): lust check accepted the template and the
// instance failed to build; a local of that type was L0640 too.
use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};
fn ext<I: IntoIterator>(i: I) -> Vec<I::Item> {
    let mut v = Vec::new();
    v.extend(i);
    v
}
fn main() {
    let m: BTreeMap<u8, String> = (1..4).map(|k| (k, k.to_string())).collect();
    let d: VecDeque<u16> = (5..8).collect();
    let s: BTreeSet<char> = "bca".chars().collect();
    println!("{:?} {:?} {:?}", ext(m.values()), ext(m.iter()), ext(d.iter()));
    println!("{:?} {:?}", ext(s.clone().into_iter()), ext(m.clone().into_keys()));
    let mut v: Vec<&String> = Vec::new();
    let vals = m.values();
    v.extend(vals);
    let mut h: HashSet<&u8> = HashSet::new();
    h.extend(m.keys());
    println!("{} {} {}", v.len(), v.capacity(), h.len());
}
