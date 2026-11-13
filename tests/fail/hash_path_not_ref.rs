// expect: L0110
// rustc E0308: Hash::hash takes `&self`.
use std::hash::{Hash, Hasher};
fn main() { let mut s = std::collections::hash_map::DefaultHasher::new(); Hash::hash(5u8, &mut s); println!("{}", s.finish()); }
