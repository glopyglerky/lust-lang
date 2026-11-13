// feature: std.hashmap_hasher
// form: HashMap with an explicit third hasher type arg, the shape behind FxHashMap aliases
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::collections::hash_map::DefaultHasher;
type FastMap<K, V> = HashMap<K, V, BuildHasherDefault<DefaultHasher>>;
fn main() {
    let mut m: FastMap<String, i32> = FastMap::default();
    m.insert("a".to_string(), 4);
    println!("{}", m.get("a").copied().unwrap_or(0));
}
