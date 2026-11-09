// rustc-hash 2.1.2: the maps case with each map built through a std
// path (HashMap::default, HashSet::with_capacity_and_hasher) and the
// Fx alias only as the type, so it checks the adapter even while lust
// cannot call an associated fn through a type alias (maps.rs does).
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use std::collections::{HashMap, HashSet};

fn main() {
    let mut m: FxHashMap<String, i32> = HashMap::default();
    for (i, w) in ["alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta"].iter().enumerate() {
        m.insert(w.to_string(), i as i32);
    }
    println!("{} {} {:?}", m.len(), m.capacity(), m.get("gamma"));
    let mut keys: Vec<&String> = Vec::new();
    for k in m.keys() {
        keys.push(k);
    }
    println!("{:?}", keys);
    let mut s: FxHashSet<u32> = HashSet::with_capacity_and_hasher(4, FxBuildHasher);
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
