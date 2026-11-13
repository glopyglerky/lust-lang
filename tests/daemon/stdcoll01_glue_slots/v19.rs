use std::collections::BTreeMap;
fn main() {
    let mut m: BTreeMap<&str, i64> = BTreeMap::new();
    let c = m.clone(); println!("{} {:?}", c == m, c.into_iter().collect::<Vec<_>>());
    println!("{:?}", m.insert("zz", -59));
    println!("{:?} {}", m, m.len());
    println!("{:?}", m.insert("m", -19));
    println!("{} {:?}", m.keys().count(), m.values().rev().take(2).collect::<Vec<_>>());
    println!("{:?}", m.insert("m", -27));
    let c = m.clone(); println!("{} {:?}", c == m, c.into_iter().collect::<Vec<_>>());
    println!("{} {:?}", m.keys().count(), m.values().rev().take(2).collect::<Vec<_>>());
    println!("{:?}", m.insert("a", -64));
    println!("{:?}", m.insert("zz", -95));
    println!("{:?}", m.insert("a", -74));
    println!("{:?} {:?}", m.get(&"a"), m.contains_key(&"zz"));
    println!("{:?} {:?}", m.first_key_value(), m.last_key_value());
    println!("{:?}", m.insert("zz", 7));
    let b: BTreeMap<&str, i64> = BTreeMap::from_iter(vec![("a", -96), ("b", 65), ("k", -72)].into_iter()); println!("{:?}", b);
}
