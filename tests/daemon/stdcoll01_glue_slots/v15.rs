use std::collections::BTreeMap;
fn main() {
    let mut m: BTreeMap<u8, i64> = BTreeMap::new();
    println!("{:?}", m.iter().map(|(k, _)| k).collect::<Vec<_>>());
    println!("{:?}", m.insert(0, 29));
    let hi = m.split_off(&3); println!("{:?} {:?}", m, hi); let mut hi = hi; m.append(&mut hi);
    println!("{:?} {:?}", m.get(&0), m.contains_key(&7));
    println!("{:?}", m.insert(10, -48));
    let mut i = 0; m.retain(|_, _| { i += 1; i % 2 == 0 }); println!("{:?}", m);
    let c = m.clone(); println!("{} {:?}", c == m, c.into_iter().collect::<Vec<_>>());
    let c = m.clone(); println!("{} {:?}", c == m, c.into_iter().collect::<Vec<_>>());
    println!("{:?}", m.iter().map(|(k, _)| k).collect::<Vec<_>>());
    let c = m.clone(); println!("{} {:?}", c == m, c.into_iter().collect::<Vec<_>>());
    let hi = m.split_off(&0); println!("{:?} {:?}", m, hi); let mut hi = hi; m.append(&mut hi);
    println!("{} {:?}", m.keys().count(), m.values().rev().take(2).collect::<Vec<_>>());
    m.entry(11).or_insert(80); println!("{:?}", m);
    let mut i = 0; m.retain(|_, _| { i += 1; i % 2 == 0 }); println!("{:?}", m);
    println!("{:?}", m.iter().map(|(k, _)| k).collect::<Vec<_>>());
}
