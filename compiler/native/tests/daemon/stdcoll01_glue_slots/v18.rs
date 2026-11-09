use std::collections::BTreeMap;
fn main() {
    let mut m: BTreeMap<&str, &str> = BTreeMap::new();
    println!("{:?}", m.iter().map(|(k, _)| k).collect::<Vec<_>>());
    println!("{:?}", m.insert("m", ""));
    let hi = m.split_off(&"m"); println!("{:?} {:?}", m, hi); let mut hi = hi; m.append(&mut hi);
    println!("{:?} {}", m, m.len());
    println!("{:?}", m.remove(&"b"));
    println!("{:?}", m.insert("b", "w"));
    println!("{:?} {:?}", m.get(&"a"), m.contains_key(&"m"));
    println!("{:?} {:?}", m.get(&"b"), m.contains_key(&"k"));
    m.entry("a").or_insert("w"); println!("{:?}", m);
}
