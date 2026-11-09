// One std source module: alloc's collections.
use std::collections::BTreeMap;

pub fn go() {
    let mut m = BTreeMap::new();
    m.insert(2u8, "b");
    m.insert(1u8, "a");
    for (k, v) in &m {
        println!("{k} {v}");
    }
}
