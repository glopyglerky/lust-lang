// A map's written `PartialEq` impl bounds both hasher params by
// `BuildHasher`, and the field's hasher comes from a defaulted type
// param (`S = RandomState`), as in indexmap. The derive glue proves
// bounds outside the derive family with the full solver
// (`inst_pred_holds`), also for bounds over a composite type and a
// user trait.
use std::collections::hash_map::RandomState;
use std::fmt;
use std::hash::{BuildHasher, Hash};

trait Tag {}
impl Tag for Vec<u8> {}

struct M<K, V, S = RandomState> {
    k: Vec<K>,
    v: Vec<V>,
    s: S,
}
impl<K: Hash + Eq, V1: PartialEq<V2>, V2, S1: BuildHasher, S2: BuildHasher> PartialEq<M<K, V2, S2>> for M<K, V1, S1> {
    fn eq(&self, o: &M<K, V2, S2>) -> bool {
        self.k == o.k && self.v.len() == o.v.len() && self.v.iter().zip(&o.v).all(|(a, b)| a == b)
    }
}
impl<K: Hash + Eq, V: Eq, S: BuildHasher> Eq for M<K, V, S> {}
impl<K: fmt::Debug, V, S> fmt::Debug for M<K, V, S>
where
    Vec<S>: Tag,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "M{:?}", self.k)
    }
}

#[derive(PartialEq, Eq)]
struct Flat {
    m: M<String, i32>,
    o: Option<M<u8, u8>>,
}
#[derive(Debug)]
struct Shown {
    m: M<u8, (), u8>,
}

fn main() {
    let mk = |v: i32| Flat { m: M { k: vec!["a".to_string()], v: vec![v], s: RandomState::new() }, o: None };
    println!("{} {}", mk(1) == mk(1), mk(1) == mk(2));
    println!("{:?}", Shown { m: M { k: vec![7], v: vec![], s: 0 } });
    let _ = &mk(0).m.s;
}
