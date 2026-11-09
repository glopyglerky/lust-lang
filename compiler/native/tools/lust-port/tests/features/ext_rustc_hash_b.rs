// feature: ext.rustc_hash
// form: FxHashSet::default, with_capacity_and_hasher(n, Default::default()), FxBuildHasher, and derived Default with Fx fields
// cargo-dep: rustc-hash = "=2.1.2"
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
#[derive(Default)]
struct Tables {
    names: FxHashMap<u32, String>,
    seen: FxHashSet<u32>,
}
fn main() {
    let mut t = Tables::default();
    t.names.insert(1, "a".to_string());
    t.seen.insert(1);
    let mut m: FxHashMap<&str, usize> = FxHashMap::with_capacity_and_hasher(8, Default::default());
    *m.entry("k").or_insert(0) += 2;
    let mut s = FxHashSet::default();
    s.insert(3u8);
    let h: FxHashSet<u8> = FxHashSet::with_hasher(FxBuildHasher);
    println!("{} {} {} {} {}", t.names[&1], t.seen.contains(&1), m["k"], s.len(), h.is_empty());
}
