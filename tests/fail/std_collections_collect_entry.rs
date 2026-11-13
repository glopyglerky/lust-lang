// expect: L0110
// rustc E0277: the items are `(u8, u16)`, the map's entries `(u8, u8)`.
use std::collections::BTreeMap;
fn main() {
    let s: BTreeMap<u8, u8> = vec![(1u8, 2u16)].into_iter().collect();
    println!("{}", s.len());
}
