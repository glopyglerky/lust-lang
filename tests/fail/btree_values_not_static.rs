// expect: L0733
// rustc: lifetime may not live long enough. The values of a borrowed map
// borrow the map, unlike its `'static` keys.
use std::collections::BTreeMap;
fn g<'m>(m: &'m BTreeMap<&'static str, u64>) -> Option<&'static u64> {
    m.values().last()
}
fn main() {
    let mut b = BTreeMap::new();
    b.insert("a", 1u64);
    println!("{:?}", g(&b));
}
