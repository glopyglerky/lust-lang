use std::collections::BTreeMap;
fn main() {
    let mut m: BTreeMap<char, i64> = BTreeMap::new();
    println!("{:?}", m.iter().map(|(k, _)| k).collect::<Vec<_>>());
    println!("{} {:?}", m.keys().count(), m.values().rev().take(2).collect::<Vec<_>>());
    println!("{:?}", m.insert('y', -84));
    m.entry('e').or_insert(-32); println!("{:?}", m);
    let b: BTreeMap<char, i64> = BTreeMap::from_iter(vec![('w', -12), ('e', -51), ('y', -11)].into_iter()); println!("{:?}", b);
    println!("{:?}", m.pop_first());
    println!("{:?}", m.remove(&'q'));
    println!("{:?}", m.insert('y', 16));
    let hi = m.split_off(&'e'); println!("{:?} {:?}", m, hi); let mut hi = hi; m.append(&mut hi);
    println!("{:?}", m.remove(&'w'));
    let mut i = 0; m.retain(|_, _| { i += 1; i % 2 == 0 }); println!("{:?}", m);
    println!("{:?}", m.pop_first());
    println!("{:?}", m.insert('e', 67));
    m.entry('q').or_insert(-27); println!("{:?}", m);
    println!("{:?}", m.insert('w', -69));
}
