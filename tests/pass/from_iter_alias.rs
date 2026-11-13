// `Alias::from_iter(..)` through type aliases of the std
// collections: plain and generic aliases, an `FxHash`-style
// alias pair with a user `BuildHasher` (as ts-rust writes it),
// a module alias by path, and aliases of the std source
// collections and of `String`.
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasher, Hasher};

#[derive(Default)]
struct F(u64);
impl Hasher for F {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, b: &[u8]) {
        for &x in b {
            self.0 = self.0.wrapping_mul(31).wrapping_add(x as u64);
        }
    }
}
#[derive(Default, Clone)]
struct FB;
impl BuildHasher for FB {
    type Hasher = F;
    fn build_hasher(&self) -> F {
        F(7)
    }
}

type FxMap<K, V> = HashMap<K, V, FB>;
type FxSet<T> = HashSet<T, FB>;
type V32 = Vec<u32>;
type MyStr = String;
type BM = std::collections::BTreeMap<u32, u32>;
type Q = std::collections::VecDeque<u32>;

mod m {
    pub type V = Vec<u32>;
}

fn main() {
    let map: FxMap<u32, u32> = FxMap::from_iter([(1, 2), (3, 4)]);
    println!("{} {:?}", map.len(), map.get(&3));
    let set: FxSet<u32> = FxSet::from_iter([1, 1, 2]);
    println!("{}", set.len());
    let set2: FxSet<u32> = FxHashSetAlias::from_iter([7]);
    println!("{}", set2.len());
    println!("{:?}", V32::from_iter([5, 6]));
    println!("{}", MyStr::from_iter(['z', 'w']));
    println!("{:?}", BM::from_iter([(9, 90)]));
    println!("{:?}", Q::from_iter([4, 5]));
    println!("{:?}", m::V::from_iter([8]));
}

type FxHashSetAlias = FxSet<u32>;
