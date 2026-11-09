use std::collections::BTreeMap;
fn main() {
    let mut m: BTreeMap<char, &str> = BTreeMap::new();
    let hi = m.split_off(&'w'); println!("{:?} {:?}", m, hi); let mut hi = hi; m.append(&mut hi);
    println!("{:?}", m.insert('e', ""));
    let c = m.clone(); println!("{} {:?}", c == m, c.into_iter().collect::<Vec<_>>());
    println!("{:?}", m.insert('r', "w"));
    m.entry('t').or_insert(""); println!("{:?}", m);
    println!("{:?}", m.insert('q', ""));
    println!("{:?} {:?}", m.get(&'q'), m.contains_key(&'w'));
    println!("{:?}", m.remove(&'e'));
    println!("{:?}", m.insert('t', "w"));
    println!("{:?}", m.insert('w', ""));
    println!("{} {:?}", m.keys().count(), m.values().rev().take(2).collect::<Vec<_>>());
    m.entry('t').or_insert(""); println!("{:?}", m);
    println!("{:?}", m.insert('e', "v"));
}
