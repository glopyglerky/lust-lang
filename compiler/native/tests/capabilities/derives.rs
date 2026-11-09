// capability: derives: Debug
// capability: derives: Clone
// capability: derives: Copy
// capability: derives: PartialEq
// capability: derives: Eq
// capability: derives: PartialOrd
// capability: derives: Ord
// capability: derives: Hash
// capability: derives: Default
// expect: ok

use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
enum K {
    #[default]
    A,
    B(u8),
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
struct P {
    x: f64,
    name: String,
}

fn main() {
    let mut v = vec![K::B(2), K::A, K::B(1)];
    v.sort();
    let k = v[1];
    let set: HashSet<K> = v.iter().copied().chain([K::A]).collect();
    let p = P { x: 1.0, name: "p".into() };
    let q = p.clone();
    println!("{:?} {:?} {} {} {} {:?}", v, k, set.len(), p == q, p < P { x: 2.0, ..P::default() }, K::default());
}
