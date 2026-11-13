// `HashMap::from([(k, v); N])` and `HashSet::from([t; N])` (W-STD-B):
// `from_iter` with `RandomState`. Only order-free facts are printed.
use std::collections::{HashMap, HashSet};

fn main() {
    let m = HashMap::from([("a", 1), ("b", 2), ("c", 3)]);
    let mut keys: Vec<_> = m.keys().copied().collect();
    keys.sort();
    println!("{:?} {} {:?}", keys, m.len(), m.get("b"));

    let s = HashSet::from([3u32, 1, 4, 1, 5, 9, 2, 6]);
    let mut v: Vec<u32> = s.iter().copied().collect();
    v.sort();
    println!("{:?} {}", v, s.len());

    // A later duplicate key replaces the value.
    let m2: HashMap<u8, char> = HashMap::from([(1, 'x'), (1, 'y')]);
    println!("{:?}", m2);

    let e: HashSet<i64> = HashSet::from([]);
    println!("{} {}", e.len(), e.capacity());
    let a = [(10i32, 20i32); 4];
    let m3 = HashMap::from(a);
    println!("{:?} {:?}", m3, a);
    let s2 = HashSet::<u8>::from([7]);
    println!("{} {}", s2.contains(&7), s2.capacity());
    let m4: HashMap<u64, f64> = HashMap::from([(1, 0.5)]);
    println!("{:?}", m4);
}
