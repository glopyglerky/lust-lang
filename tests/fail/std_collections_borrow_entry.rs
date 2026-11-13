// expect: L0303
// rustc E0499: the entry holds the map mutably.
use std::collections::BTreeMap;
fn main() {
    let mut m: BTreeMap<u8, u8> = BTreeMap::new();
    let e = m.entry(1);
    m.insert(2, 2);
    e.or_insert(3);
}
