// expect: L0104
// Valid Rust (`RangeFull` derives Hash and hashes nothing); lust stores a
// dummy flag for `..` in C, so it has no Hash glue and rejects.
use std::hash::{Hash, Hasher};
fn main() { let mut s = std::collections::hash_map::DefaultHasher::new(); (..).hash(&mut s); println!("{}", s.finish()); }
