mod fx;

use fx::{FxBuildHasher, FxHashMap, FxHashSet};
use std::hash::BuildHasher;

#[derive(PartialEq, Eq, Hash)]
struct Key {
    id: u32,
    name: String,
}

fn main() {
    let mut m: FxHashMap<u64, u32> = FxHashMap::default();
    m.insert(3, 4);
    println!("{} {:?}", m.contains_key(&3), m.get(&5));
    let mut names: FxHashMap<String, usize> = FxHashMap::with_capacity_and_hasher(4, Default::default());
    names.insert("a".to_string(), 1);
    *names.entry("a".to_string()).or_insert(0) += 1;
    println!("{:?} {}", names.get("a"), names["a"]);
    let mut s = FxHashSet::default();
    s.insert(Key { id: 1, name: "x".to_string() });
    println!("{}", s.contains(&Key { id: 1, name: "x".to_string() }));
    let b = FxBuildHasher;
    println!("{}", b.hash_one(7u64) == b.hash_one(7u64));
}
