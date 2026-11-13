// expect: L0110
// rustc E0599: `HashMap::new` exists only for S = RandomState.
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, DefaultHasher};
type FxHashMap<K, V> = HashMap<K, V, BuildHasherDefault<DefaultHasher>>;
fn main() {
    let m: FxHashMap<u8, u8> = FxHashMap::new();
    println!("{}", m.len());
}
