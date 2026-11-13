// Maps and sets as IntoIterator arguments: flatten, chain, zip, and eq
// take `&HashSet`, `&HashMap`, `&mut HashMap`, and the tables by value.
use std::collections::{HashMap as HM, HashSet as HS};
use std::hash::{BuildHasherDefault, DefaultHasher};
type HashMap<K, V> = HM<K, V, BuildHasherDefault<DefaultHasher>>;
type HashSet<K> = HS<K, BuildHasherDefault<DefaultHasher>>;
fn names(dirty: Option<&HashSet<String>>) -> Vec<String> {
    let mut out = Vec::new();
    for n in dirty.into_iter().flatten() { out.push(n.clone()); }
    out
}
fn main() {
    let s: HashSet<String> = ["b", "a", "c"].iter().map(|x| x.to_string()).collect();
    println!("{:?} {:?}", names(Some(&s)), names(None));
    let sets = vec![s.clone(), HashSet::default()];
    println!("{:?}", sets.iter().flatten().collect::<Vec<_>>());
    println!("{:?}", sets.into_iter().flatten().collect::<Vec<_>>());
    let mut m: HashMap<u8, u8> = HashMap::default();
    for i in 0..6 { m.insert(i, i * 2); }
    println!("{:?}", vec![1u8].into_iter().chain(m.values().copied()).collect::<Vec<_>>());
    println!("{:?}", (0..3).zip(&m).collect::<Vec<_>>());
    let ms = vec![m.clone(), m];
    println!("{:?}", ms.iter().flatten().map(|(k, v)| k + v).collect::<Vec<_>>());
    let mut mm = ms;
    for (_, v) in mm.iter_mut().flatten() { *v += 1; }
    println!("{:?}", mm[0].values().collect::<Vec<_>>());
    println!("{}", s.iter().eq(&s));
}
