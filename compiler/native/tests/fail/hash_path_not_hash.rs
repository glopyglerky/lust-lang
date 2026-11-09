// expect: L0612
// rustc E0277: the receiver of Hash::hash must be Hash.
use std::hash::{Hash, Hasher};
struct N;
fn main() { let mut s = std::collections::hash_map::DefaultHasher::new(); Hash::hash(&N, &mut s); println!("{}", s.finish()); }
