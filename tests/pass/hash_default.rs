// Default for the hash types (W-STD-B): Default::default() for maps, sets,
// and hashers, #[derive(Default)] with map and set fields, mem::take,
// a nested or_default, and a hasher argument typed by the let.
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasher, BuildHasherDefault, DefaultHasher, Hasher, RandomState};

#[derive(Default, Debug)]
struct Cache {
    map: HashMap<String, u32>,
    seen: HashSet<u8>,
    hits: u32,
}

fn main() {
    let mut m: HashMap<u8, u8> = Default::default();
    m.insert(1, 2);
    let s: HashSet<String> = Default::default();
    println!("{} {} {}", m[&1], s.len(), s.is_empty());
    let r: RandomState = Default::default();
    let _ = r.hash_one(5u8);
    let mut h: DefaultHasher = Default::default();
    h.write_u8(1);
    println!("{}", h.finish());
    let b: BuildHasherDefault<DefaultHasher> = Default::default();
    println!("{}", b.hash_one(7u32));
    let mut fixed: HashMap<u8, u8, BuildHasherDefault<DefaultHasher>> = HashMap::with_capacity_and_hasher(4, Default::default());
    fixed.insert(3, 4);
    fixed.insert(5, 6);
    let mut keys: Vec<_> = fixed.keys().copied().collect();
    keys.sort();
    println!("{:?} {}", keys, fixed[&5]);
    let mut c = Cache::default();
    c.map.insert("a".into(), 1);
    c.hits += 1;
    println!("{:?} {} {}", c.map, c.seen.len(), c.hits);
    let taken = std::mem::take(&mut c.map);
    println!("{} {}", taken.len(), c.map.len());
    let pair: (HashMap<u8, u8>, u8) = Default::default();
    println!("{} {}", pair.0.len(), pair.1);
    let mut nested: HashMap<u8, HashMap<u8, u8>> = HashMap::new();
    nested.entry(1).or_default().insert(2, 3);
    println!("{:?}", nested);
}
