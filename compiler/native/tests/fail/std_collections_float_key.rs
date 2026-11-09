// expect: L0612
// rustc E0277: `f64` is not `Ord`, so it cannot be a BTreeMap key.
use std::collections::BTreeMap;
fn main() {
    let mut m = BTreeMap::new();
    m.insert(1.5f64, 1);
}
