// `Default` builds the hash tables and builders by structure: derived
// Default with map fields, and `Default::default()` at a table type.
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, DefaultHasher, RandomState};
type FxHashMap<K, V> = HashMap<K, V, BuildHasherDefault<DefaultHasher>>;
#[derive(Default)]
struct Tables {
    plain: HashMap<u8, u8>,
    fx: FxHashMap<String, u32>,
    set: HashSet<i64, BuildHasherDefault<DefaultHasher>>,
    n: u8,
}
fn main() {
    let mut t = Tables::default();
    t.fx.insert(String::from("a"), 1);
    t.set.insert(-3);
    println!("{} {} {} {}", t.plain.len(), t.fx["a"], t.set.contains(&-3), t.n);
    let m: FxHashMap<u8, u8> = Default::default();
    let r: RandomState = Default::default();
    let mut n: HashMap<u8, u8> = HashMap::with_hasher(r);
    n.insert(1, 2);
    println!("{} {:?}", m.len(), n.get(&1));
}
