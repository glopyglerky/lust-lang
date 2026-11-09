// expect: L0104
// rustc E0599: core implements Hash for tuples of at most 12 elements, so a
// 13-tuple has no `hash` method.
use std::hash::{Hash, Hasher};
fn main() {
    let t = (0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8);
    let mut h = std::collections::hash_map::DefaultHasher::new();
    t.hash(&mut h);
    println!("{}", h.finish());
}
