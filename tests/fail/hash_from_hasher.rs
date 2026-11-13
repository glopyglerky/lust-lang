// expect: L0612
// rustc E0308: `From<[(K, V); N]>` builds a map with `RandomState` only.
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, DefaultHasher};

fn main() {
    let m = HashMap::<u8, u8, BuildHasherDefault<DefaultHasher>>::from([(1, 2)]);
    println!("{m:?}");
}
