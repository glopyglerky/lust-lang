// expect: L0303
// rustc E0502: `r` borrows the map across the remove.
use std::collections::BTreeMap;
fn main() {
    let mut m = BTreeMap::new();
    m.insert(1, String::from("a"));
    let r = m.get(&1).unwrap();
    m.remove(&1);
    println!("{}", r);
}
