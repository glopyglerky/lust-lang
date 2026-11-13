// expect: L0112
// rustc E0369: the field type's generic `PartialEq` impl needs
// `K: Eq`, and `f64` is not `Eq`.
use std::hash::Hash;
struct IM<K, V, S> {
    entries: Vec<(K, V)>,
    s: S,
}
impl<K: Eq + Hash, V: PartialEq, S, S2> PartialEq<IM<K, V, S2>> for IM<K, V, S> {
    fn eq(&self, o: &IM<K, V, S2>) -> bool {
        self.entries.len() == o.entries.len()
    }
}
#[derive(PartialEq)]
struct W {
    m: IM<f64, u32, u8>,
}
fn main() {
    println!("x");
}
