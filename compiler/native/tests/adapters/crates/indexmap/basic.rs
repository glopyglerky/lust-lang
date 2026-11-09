// indexmap 2.14.0: insertion order, lookups, both removal orders, the
// entry API, sorting, capacity, and Debug, as ts-rust uses IndexMap.
use indexmap::{IndexMap, IndexSet};

fn main() {
    let mut m: IndexMap<String, u32> = IndexMap::new();
    for (i, w) in ["delta", "alpha", "charlie", "bravo", "echo", "alpha"].iter().enumerate() {
        m.insert(w.to_string(), i as u32);
    }
    println!("{:?} len {} cap {}", m, m.len(), m.capacity());
    println!("{:?} {:?} {:?}", m.get("charlie"), m.get_index_of("bravo"), m.get_full("echo"));
    println!("{:?} {:?}", m.first(), m.last());
    println!("{:?}", m.shift_remove("alpha"));
    println!("{:?}", m.swap_remove("delta"));
    println!("{:?}", m);
    *m.entry("foxtrot".to_string()).or_insert(0) += 10;
    *m.entry("charlie".to_string()).or_insert(0) += 10;
    println!("{:?}", m);
    m.sort_keys();
    let keys: Vec<&String> = m.keys().collect();
    println!("{:?} {:?}", keys, m.values().rev().collect::<Vec<_>>());
    m.retain(|k, v| k.len() > 4 || *v > 5);
    println!("{:?} cap {}", m, m.capacity());
    let mut s: IndexSet<u32> = IndexSet::new();
    for x in [5u32, 3, 5, 9, 1, 3] {
        s.insert(x);
    }
    println!("{:?} {} {:?}", s, s.contains(&9), s.get_index_of(&1));
}
