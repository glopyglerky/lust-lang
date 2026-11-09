// `collect` into a HashMap or HashSet (W-STD-B): the table holds the
// items' regions, not the iterator's borrow of its source.
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, DefaultHasher};
type B = BuildHasherDefault<DefaultHasher>;
fn main() {
    let s: HashSet<&str, B>;
    let m: HashMap<u8, &str, B>;
    {
        let v = vec!["a", "b"];
        s = v.iter().copied().collect();
        m = v.iter().map(|k| (k.len() as u8, *k)).collect();
    }
    let t: HashSet<&str, B> = ["x", "y"].iter().copied().collect();
    println!("{s:?} {m:?} {t:?}");
}
