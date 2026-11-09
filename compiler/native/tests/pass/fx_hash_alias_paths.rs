// The rustc-hash shape: generic aliases of HashMap and HashSet with a
// BuildHasherDefault hasher, called through the alias, `<T>::`, and a
// module path; `Default::default()` as the hasher argument.
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};

#[derive(Default)]
pub struct FxHasher {
    hash: u64,
}
impl Hasher for FxHasher {
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.hash = (self.hash.rotate_left(5) ^ b as u64).wrapping_mul(0x51_7c_c1_b7_27_22_0a_95);
        }
    }
    fn finish(&self) -> u64 {
        self.hash
    }
}
pub type FxBuildHasher = BuildHasherDefault<FxHasher>;
pub type FxHashMap<K, V> = HashMap<K, V, FxBuildHasher>;
pub type FxHashSet<V> = HashSet<V, FxBuildHasher>;
mod m {
    pub type Map<K, V> = std::collections::HashMap<K, V, super::FxBuildHasher>;
}

fn main() {
    let mut a: FxHashMap<u32, String> = FxHashMap::default();
    a.insert(1, String::from("a"));
    a.insert(2, String::from("b"));
    let mut keys: Vec<_> = a.keys().copied().collect();
    keys.sort();
    println!("{:?} {}", keys, a[&2]);
    let mut s = FxHashSet::default();
    s.insert("x");
    s.insert("x");
    println!("{}", s.len());
    let c: FxHashMap<u8, u8> = FxHashMap::with_capacity_and_hasher(8, Default::default());
    println!("{} {}", c.len(), c.capacity() >= 8);
    let d: HashMap<u8, u8, FxBuildHasher> = HashMap::with_capacity_and_hasher(4, Default::default());
    let e: FxHashSet<u16> = FxHashSet::with_hasher(FxBuildHasher::default());
    let f = <FxHashMap<i32, i32>>::default();
    let g = <HashMap<u8, u8>>::new();
    let mut h: m::Map<String, u8> = m::Map::default();
    h.insert(String::from("k"), 4);
    println!("{} {} {} {} {}", d.is_empty(), e.len(), f.len(), g.len(), h["k"]);
    let b: FxBuildHasher = Default::default();
    let mut i: FxHashMap<u8, Vec<u8>> = FxHashMap::with_hasher(b);
    i.entry(1).or_default().push(2);
    println!("{:?}", i[&1]);
}
