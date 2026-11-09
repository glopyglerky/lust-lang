use std::collections::BTreeMap;
fn main() {
    let mut m: BTreeMap<char, Vec<u8>> = BTreeMap::new();
    println!("{:?}", m.remove(&'t'));
    println!("{} {:?}", m.keys().count(), m.values().rev().take(2).collect::<Vec<_>>());
    let c = m.clone(); println!("{} {:?}", c == m, c.into_iter().collect::<Vec<_>>());
    let hi = m.split_off(&'y'); println!("{:?} {:?}", m, hi); let mut hi = hi; m.append(&mut hi);
    let b: BTreeMap<char, Vec<u8>> = BTreeMap::from_iter(vec![('w', vec![1]), ('e', vec![0]), ('e', vec![3])].into_iter()); println!("{:?}", b);
    println!("{:?}", m.pop_first());
    let c = m.clone(); println!("{} {:?}", c == m, c.into_iter().collect::<Vec<_>>());
    println!("{:?} {:?}", m.get(&'w'), m.contains_key(&'r'));
    println!("{:#?} {}", m, m.len());
    println!("{:?}", m.insert('t', vec![3]));
    println!("{:?} {:?}", m.first_key_value(), m.last_key_value());
    println!("{:?}", m.pop_first());
    let b: BTreeMap<char, Vec<u8>> = BTreeMap::from_iter(vec![('e', vec![3]), ('t', vec![2]), ('q', vec![0])].into_iter()); println!("{:?}", b);
}
