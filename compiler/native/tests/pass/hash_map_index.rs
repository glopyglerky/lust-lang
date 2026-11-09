// `map[&q]` (W-STD-B): std's `Index<&Q>` for HashMap, with `get`'s Borrow
// lookups (`m["a"]` on `&str` keys, `n["x"]` on String keys), through
// references, in closures, nested maps, and a later insert.
use std::collections::HashMap;

fn main() {
    let mut m = HashMap::new();
    m.insert("a", 1);
    m.insert("b", 2);
    println!("{} {}", m["a"], m["b"] + 10);
    let mut n: HashMap<String, Vec<u8>> = HashMap::new();
    let k = String::from("x");
    n.insert(k.clone(), vec![1, 2, 3]);
    println!("{} {:?} {}", n[&k].len(), n["x"], n[&k][1]);
    let r = &n["x"];
    println!("{:?}", r.iter().rev().collect::<Vec<_>>());
    let f = |key: &str| n[key].len();
    println!("{}", f("x"));
    let total: u32 = ["a", "b"].iter().map(|s| m[s]).sum();
    println!("{}", total);
    let mut nested: HashMap<u8, HashMap<u8, char>> = HashMap::new();
    nested.insert(1, HashMap::from([(2, 'z')]));
    println!("{}", nested[&1][&2]);
    let mr = &m;
    println!("{}", mr["b"]);
    m.insert("c", 3);
    println!("{}", m["c"]);
}
