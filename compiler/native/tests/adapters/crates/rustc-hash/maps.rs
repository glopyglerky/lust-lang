// rustc-hash 2.1.2: FxHashMap and FxHashSet through the aliases, as
// ts-rust builds them (default, with_capacity_and_hasher, with_hasher),
// with iteration order and capacity, which the Fx hashes fix.
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use std::collections::HashMap;

fn main() {
    let mut m: FxHashMap<String, i32> = FxHashMap::default();
    for (i, w) in ["alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta"].iter().enumerate() {
        m.insert(w.to_string(), i as i32);
    }
    println!("{} {} {:?}", m.len(), m.capacity(), m.get("gamma"));
    let mut keys: Vec<&String> = Vec::new();
    for k in m.keys() {
        keys.push(k);
    }
    println!("{:?}", keys);
    let mut s: FxHashSet<u32> = FxHashSet::with_capacity_and_hasher(4, FxBuildHasher);
    for i in [5u32, 1, 9, 13, 2, 40, 7] {
        s.insert(i);
    }
    let mut v: Vec<u32> = Vec::new();
    for x in &s {
        v.push(*x);
    }
    println!("{:?} {}", v, s.capacity());
    let mut h: HashMap<u64, u64, FxBuildHasher> = HashMap::with_hasher(FxBuildHasher);
    for i in 0..20u64 {
        h.insert(i * 7, i);
    }
    h.remove(&14);
    let mut order: Vec<u64> = Vec::new();
    for (k, _) in &h {
        order.push(*k);
    }
    println!("{:?} {}", order, h.capacity());
}
