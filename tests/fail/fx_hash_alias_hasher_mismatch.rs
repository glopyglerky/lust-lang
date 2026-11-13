// expect: L0110
// rustc E0308: the alias fixes S = BuildHasherDefault<DefaultHasher>.
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, DefaultHasher, RandomState};
type FxHashMap<K, V> = HashMap<K, V, BuildHasherDefault<DefaultHasher>>;
fn main() {
    let m: FxHashMap<u8, u8> = FxHashMap::with_capacity_and_hasher(8, RandomState::new());
    println!("{}", m.len());
}
