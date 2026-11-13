// indexmap 2.14.0: clone of the borrowing iterators (`Iter`, `Keys`,
// `Values`, `set::Iter`) restarts from the same position, as
// ts-rust's resolver walks `path_mappings.keys()` twice.
use indexmap::{IndexMap, IndexSet};

fn main() {
    let mut m: IndexMap<String, Option<Vec<String>>> = IndexMap::new();
    for w in ["b/*", "a", "c/*", "d"] {
        m.insert(w.to_string(), Some(vec![w.to_string()]));
    }
    let paths = m.keys();
    let mut n = 0;
    for p in paths.clone() {
        if p.contains('*') {
            n += 1;
        }
    }
    println!("{} {:?}", n, paths.collect::<Vec<_>>());
    let mut it = m.iter();
    it.next();
    let c = it.clone();
    println!("{:?} {}", c.map(|(k, _)| k.as_str()).collect::<Vec<_>>(), it.count());
    let vs = m.values();
    println!("{} {}", vs.clone().flatten().count(), vs.len());
    let s: IndexSet<u8> = [5, 3, 9, 1].into_iter().collect();
    let mut si = s.iter();
    si.next_back();
    let sc = si.clone();
    println!("{:?} {:?}", sc.collect::<Vec<_>>(), si.rev().collect::<Vec<_>>());
}
