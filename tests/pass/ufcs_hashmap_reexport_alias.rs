// Builtin methods by type-relative path through a std re-export (use std::collections::HashMap)
// and through a generic alias of HashMap with its hasher (ts-rust: FxHashMap::clone(&m)).
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, DefaultHasher};
type FxMap<K, V> = HashMap<K, V, BuildHasherDefault<DefaultHasher>>;
fn copy_of(shared: &FxMap<u32, String>) -> FxMap<u32, String> {
    FxMap::clone(shared)
}
fn main() {
    let mut src = FxMap::default();
    src.insert(1, "a".to_string());
    let m = copy_of(&src);
    let n = std::collections::HashMap::clone(&src);
    let len = FxMap::len(&m);
    let plain: HashMap<u8, u8> = HashMap::from([(1, 2)]);
    let p = HashMap::clone(&plain);
    let v = Vec::<u8>::clone(&vec![1]);
    println!("{:?} {:?} {} {:?} {:?}", m, n, len, p, v);
}
