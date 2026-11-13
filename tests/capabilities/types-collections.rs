// capability: types: HashMap<K, V> and HashSet<T>
// capability: types: std::collections::{BTreeMap, BTreeSet, VecDeque} with btree_map::Entry, owned-item Extend, From arrays, Index and range, and collect into all three (std source; keys with derived or std order only)
// expect: ok

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

fn main() {
    let mut h: HashMap<String, u32> = HashMap::new();
    *h.entry("a".into()).or_insert(0) += 2;
    let hs: HashSet<u32> = [1, 2, 2].into_iter().collect();
    let mut bm: BTreeMap<u32, u32> = BTreeMap::from([(3, 30), (1, 10)]);
    *bm.entry(2).or_insert(0) += 20;
    bm.extend([(4, 40)]);
    let r: Vec<_> = bm.range(2..4).collect();
    let bs: BTreeSet<char> = "hello".chars().collect();
    let dq: VecDeque<u32> = (0..3).collect();
    println!("{} {} {} {:?} {:?} {:?}", h["a"], hs.len(), bm[&2], r, bs, dq);
}
