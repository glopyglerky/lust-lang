// Without `std`, a map takes an explicit hasher.
use std::collections::hash_map::RandomState;

fn main() {
    let mut m: indexmap::IndexMap<i32, i32, RandomState> = indexmap::IndexMap::with_hasher(RandomState::new());
    m.insert(1, 2);
    println!("{:?}", m.get(&1));
}
