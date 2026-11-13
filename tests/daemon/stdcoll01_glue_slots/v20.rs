use std::collections::BTreeMap;
fn main() {
    let mut m: BTreeMap<(u8, bool), i64> = BTreeMap::new();
    println!("{:?}", m.iter().map(|(k, _)| k).collect::<Vec<_>>());
    println!("{:?} {:?}", m.first_key_value(), m.last_key_value());
    println!("{:?} {:?}", m.get(&(3, true)), m.contains_key(&(1, false)));
    let hi = m.split_off(&(0, true)); println!("{:?} {:?}", m, hi); let mut hi = hi; m.append(&mut hi);
    println!("{:?}", m.insert((1, true), -66));
    println!("{} {:?}", m.keys().count(), m.values().rev().take(2).collect::<Vec<_>>());
    println!("{} {:?}", m.keys().count(), m.values().rev().take(2).collect::<Vec<_>>());
    let c = m.clone(); println!("{} {:?}", c == m, c.into_iter().collect::<Vec<_>>());
    let hi = m.split_off(&(0, false)); println!("{:?} {:?}", m, hi); let mut hi = hi; m.append(&mut hi);
    println!("{:?}", m.insert((3, false), -83));
    println!("{} {:?}", m.keys().count(), m.values().rev().take(2).collect::<Vec<_>>());
    println!("{:?}", m.insert((0, false), -11));
}
