// expect: L0612
// rustc E0277: HashMap<K, V, S>: Default needs S: Default. With
// trait-name calls deferred to body end, the obligation names
// the missing impl (it was an L0101 on `Default` before).
use std::collections::HashMap;
use std::hash::{BuildHasher, DefaultHasher};

struct NoDefault;

impl BuildHasher for NoDefault {
    type Hasher = DefaultHasher;
    fn build_hasher(&self) -> DefaultHasher {
        DefaultHasher::new()
    }
}

fn main() {
    let m: HashMap<u8, u8, NoDefault> = Default::default();
    println!("{}", m.len());
}
