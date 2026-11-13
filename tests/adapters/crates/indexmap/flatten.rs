// indexmap 2.14.0: flatten over IndexMap and IndexSet items, by &, &mut, and value (IntoIterator per item, R15).
use indexmap::{IndexMap, IndexSet};
fn main() {
    let mut m = IndexMap::new(); m.insert("a".to_string(), "x".to_string()); m.insert("b".to_string(), "y".to_string());
    let d: Option<IndexMap<String, String>> = Some(m);
    let mut files: Vec<String> = Vec::new();
    for (_p, name) in d.iter().flatten() { files.push(name.clone()); }
    println!("{:?}", files);
    let n: Option<IndexMap<String, String>> = None;
    println!("{}", n.iter().flatten().count());
    let mut it = d.iter().flatten();
    println!("{:?} {:?} {:?} {:?}", it.size_hint(), it.next(), it.size_hint(), it.next());
    let mut maps = vec![IndexMap::new(), IndexMap::new()]; maps[1].insert(5u8, 6u8);
    for (k, v) in maps.iter_mut().flatten() { *v += *k; }
    println!("{:?}", maps);
    let owned: Vec<(u8, u8)> = maps.into_iter().flatten().collect();
    println!("{:?}", owned);
    let mut s1: IndexSet<i32> = IndexSet::new(); s1.insert(3); s1.insert(1);
    let v = vec![s1.clone(), s1];
    let all: Vec<&i32> = v.iter().flatten().collect();
    println!("{:?}", all);
    let ints: Vec<i32> = v.into_iter().flatten().collect();
    println!("{:?}", ints);
}
