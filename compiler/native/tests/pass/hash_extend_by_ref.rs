// `Extend<&T>` for HashSet<T: Copy> and `Extend<(&K, &V)>` for HashMap
// (W-STD-B): the items copied in, with std's reserve rule.
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, DefaultHasher};
type B = BuildHasherDefault<DefaultHasher>;
fn main() {
    let mut s: HashSet<u32, B> = HashSet::default();
    s.extend(&[3u32, 1, 3]);
    let v = vec![7u32, 8];
    s.extend(v.iter());
    s.extend(&v);
    let mut m: HashMap<u8, char, B> = HashMap::default();
    let src: HashMap<u8, char, B> = [(1u8, 'a'), (2, 'b')].into_iter().collect();
    m.extend(&src);
    m.extend(src.iter().map(|(k, v)| (k, v)));
    println!("{s:?} {} {m:?} {} {:?}", s.capacity(), m.capacity(), v);
    // Owned items with a reference key stay by value.
    let mut r: HashMap<&str, i32, B> = HashMap::default();
    r.extend(vec![("x", 1), ("y", 2)]);
    let mut t: HashSet<&str, B> = HashSet::default();
    let words = vec!["p", "q"];
    t.extend(words.clone());
    t.extend(&words);
    let mut u: HashMap<&str, &str, B> = HashMap::default();
    {
        // The borrow of `tmp` ends with the call; the keys stay.
        let tmp: HashMap<&str, &str, B> = r.keys().map(|k| (*k, *k)).collect();
        u.extend(&tmp);
    }
    println!("{r:?} {t:?} {u:?}");
}
