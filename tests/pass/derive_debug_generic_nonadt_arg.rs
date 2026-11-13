// A user `impl<K: Debug> Debug for W<K>` must prove under derive
// for a generic argument that is not an Adt path type (a reference,
// a tuple, an array), and inside another generic Debug body at its
// instance (`Map<u8, Set<u8>>` formats `&Set<u8>` items). The bench
// IndexMap shape (W-BENCH4 rank 1).
use std::fmt;
pub struct W<K>(K);
impl<K: fmt::Debug> fmt::Debug for W<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) }
}
pub struct Set<T>(Vec<T>);
impl<T: fmt::Debug> fmt::Debug for Set<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.debug_set().entries(self.0.iter()).finish() }
}
pub struct Map<K, V>(Vec<(K, V)>);
impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for Map<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.debug_map().entries(self.0.iter().map(|(k, v)| (k, v))).finish() }
}
#[derive(Debug)]
struct A { m: W<&'static u8>, t: W<(u8, [u8; 2])> }
fn main() {
    println!("{:?}", A { m: W(&7), t: W((1, [2, 3])) });
    println!("{:?}", Map(vec![(1u8, Set(vec![2u8]))]));
    println!("{:?}", &Set(vec![3u8]));
}
