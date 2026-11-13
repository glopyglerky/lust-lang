// The slice's `IndexMap` shape: `impl<K: Eq + Hash, V: PartialEq,
// S, S2> PartialEq<IndexMap<K, V, S2>>` plus a generic written
// `Debug`. The `Eq` bound on the key is part of what the derive
// glue must prove (`glue_total_eq`).
use std::hash::Hash;
struct IM<K, V, S> {
    entries: Vec<(K, V)>,
    s: S,
}
impl<K: Eq + Hash, V: PartialEq, S, S2> PartialEq<IM<K, V, S2>> for IM<K, V, S> {
    fn eq(&self, o: &IM<K, V, S2>) -> bool {
        self.entries.len() == o.entries.len()
            && self.entries.iter().all(|(k, v)| o.entries.iter().any(|(ok, ov)| ok == k && ov == v))
    }
}
impl<K: std::fmt::Debug, V: std::fmt::Debug, S> std::fmt::Debug for IM<K, V, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IM({})", self.entries.len())
    }
}
#[derive(PartialEq, Debug)]
struct W {
    m: IM<String, u32, u8>,
}
fn main() {
    let a = W { m: IM { entries: vec![("k".to_string(), 1)], s: 0 } };
    let b = W { m: IM { entries: vec![("k".to_string(), 1)], s: 9 } };
    println!("{} {:?}", a == b, a);
}
