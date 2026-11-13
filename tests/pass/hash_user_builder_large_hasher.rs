// A user BuildHasher whose Hasher is DefaultHasher (a hasher C returns
// through an out pointer): map and set order, capacity, hash_one (W-STD-B).
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasher, DefaultHasher};
#[derive(Clone, Default)]
pub struct St;
impl BuildHasher for St {
    type Hasher = DefaultHasher;
    fn build_hasher(&self) -> DefaultHasher {
        DefaultHasher::new()
    }
}
fn main() {
    let mut m: HashMap<u32, &str, St> = HashMap::default();
    for (i, w) in ["a", "b", "c", "d", "e", "f"].iter().enumerate() {
        m.insert(i as u32 * 7, w);
    }
    println!("{:?} {}", m, m.capacity());
    let s: HashSet<String, St> = ["x", "yy", "zzz"].iter().map(|s| s.to_string()).collect();
    println!("{:?} {}", s, St.hash_one("k"));
}
