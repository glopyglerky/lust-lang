// expect: L0610
// rustc E0282: BTreeMap has Extend<(K, V)> and Extend<(&K, &V)>.
use std::collections::BTreeMap;
fn main() {
    let mut src = BTreeMap::new();
    src.insert(1u8, 2u8);
    let mut v = BTreeMap::new();
    v.extend(src.iter());
    println!("{}", v.len());
}
