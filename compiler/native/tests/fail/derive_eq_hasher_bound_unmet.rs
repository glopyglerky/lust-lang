// expect: L0112
// rustc E0369: the written `PartialEq` needs `S: BuildHasher`, and the
// field's hasher param is a type with no `BuildHasher` impl, so the
// derived `PartialEq` cannot compare the field.
use std::hash::BuildHasher;
struct NoBuild;
struct M<K, S = NoBuild> {
    k: Vec<K>,
    s: S,
}
impl<K: PartialEq, S1: BuildHasher, S2: BuildHasher> PartialEq<M<K, S2>> for M<K, S1> {
    fn eq(&self, o: &M<K, S2>) -> bool {
        self.k == o.k
    }
}
#[derive(PartialEq)]
struct Flat {
    m: M<u8>,
}
fn main() {
    let a = Flat { m: M { k: vec![1], s: NoBuild } };
    let _ = a.m.s;
}
