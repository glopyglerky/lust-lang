// expect: L0303
// rustc E0502: the map is borrowed by its iterator.
use std::collections::BTreeMap;
fn main() {
    let mut m = BTreeMap::new();
    m.insert(1, 1);
    for (k, _) in m.iter() {
        m.insert(*k + 10, 0);
    }
}
