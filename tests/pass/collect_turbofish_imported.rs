// A method turbofish names imported types (`collect::<HashMap<_, _>>()`).
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, DefaultHasher};
type B = BuildHasherDefault<DefaultHasher>;
fn main() {
    let m = [(1u8, 'a'), (2, 'b')].into_iter().collect::<HashMap<_, _, B>>();
    let s = ["x", "y", "x"].iter().copied().collect::<HashSet<&str, B>>();
    let n = m.keys().map(|k| *k as u32).collect::<HashSet<u32, B>>().len();
    println!("{m:?} {s:?} {n}");
}
