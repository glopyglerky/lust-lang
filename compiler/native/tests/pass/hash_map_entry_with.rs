// The Entry closure forms (W-STD-B): the closure runs only on the
// branch std runs it on.
use std::collections::HashMap;

fn main() {
    let mut m: HashMap<&str, Vec<u32>> = HashMap::new();
    let mut calls = 0;
    for (k, v) in [("a", 1), ("b", 2), ("a", 3)] {
        m.entry(k)
            .or_insert_with(|| {
                calls += 1;
                Vec::new()
            })
            .push(v);
    }
    let mut keys: Vec<_> = m.iter().collect();
    keys.sort();
    println!("{keys:?} {calls}");
    let mut counts: HashMap<String, usize> = HashMap::new();
    for w in ["x", "y", "x", "z", "x"] {
        counts.entry(w.to_string()).and_modify(|c| *c += 1).or_insert(1);
    }
    let mut cs: Vec<_> = counts.into_iter().collect();
    cs.sort();
    println!("{cs:?}");
    let mut lens: HashMap<String, usize> = HashMap::new();
    for w in ["abc", "de", "abc"] {
        let n = lens.entry(w.to_string()).or_insert_with_key(|k| k.len() * 10);
        *n += 1;
    }
    let mut ls: Vec<_> = lens.into_iter().collect();
    ls.sort();
    println!("{ls:?}");
    let mut mods = 0;
    let mut z: HashMap<u8, u8> = HashMap::new();
    z.entry(1).and_modify(|v| {
        mods += 1;
        *v += 1;
    });
    z.insert(1, 5);
    z.entry(1).and_modify(|v| {
        mods += 1;
        *v *= 2;
    });
    println!("{:?} {mods}", z.get(&1));
    // Reference keys: the Entry's loan does not outlive the statement.
    let mut refs: HashMap<&str, Vec<u8>> = HashMap::new();
    let k = String::from("q");
    refs.entry(&k).or_insert(Vec::new()).push(1);
    refs.entry("r").or_insert_with(|| vec![9]).push(2);
    println!("{} {:?}", refs.len(), refs.get("q"));
}
