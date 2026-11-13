// indexmap 2.14.0: IndexMap::as_slice and map::Slice (len, get_index, first, last, iter, keys, values, Debug, ordered ==), as ts-rust options.rs compares paths (R15).
use indexmap::IndexMap;
fn main() {
    let mut m: IndexMap<String, Option<Vec<String>>> = IndexMap::new();
    m.insert("a".into(), Some(vec!["x".into()]));
    m.insert("b".into(), None);
    let mut n = m.clone();
    let a: Option<&IndexMap<String, Option<Vec<String>>>> = Some(&m);
    println!("{}", a.map(IndexMap::as_slice) == a.map(IndexMap::as_slice));
    let s = m.as_slice();
    println!("{} {} {:?} {:?} {:?}", s.len(), s.is_empty(), s.get_index(1), s.first(), s.last());
    println!("{:?}", s);
    println!("{:?}", s.keys().collect::<Vec<_>>());
    println!("{:?}", s.values().count());
    for (k, v) in s { println!("{k} {v:?}"); }
    n.swap_indices(0, 1);
    println!("{} {}", m == n, m.as_slice() == n.as_slice());
    let e: IndexMap<u8, u8> = IndexMap::new();
    println!("{:?} {}", e.as_slice(), e.as_slice().is_empty());
}
