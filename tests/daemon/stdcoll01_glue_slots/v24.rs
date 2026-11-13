use std::collections::BTreeMap;
fn main() {
    let mut m: BTreeMap<u8, i64> = BTreeMap::new();
    let hi = m.split_off(&9); println!("{:?} {:?}", m, hi); let mut hi = hi; m.append(&mut hi);
    m.entry(4).or_insert(91); println!("{:?}", m);
    let c = m.clone(); println!("{} {:?}", c == m, c.into_iter().collect::<Vec<_>>());
    println!("{:?}", m.pop_first());
    let hi = m.split_off(&2); println!("{:?} {:?}", m, hi); let mut hi = hi; m.append(&mut hi);
    let c = m.clone(); println!("{} {:?}", c == m, c.into_iter().collect::<Vec<_>>());
    println!("{:?} {:?}", m.get(&5), m.contains_key(&5));
    println!("{:?}", m.remove(&0));
    m.entry(10).or_insert(83); println!("{:?}", m);
    let b: BTreeMap<u8, i64> = BTreeMap::from_iter(vec![(3, 40), (7, 54), (9, 45)].into_iter()); println!("{:?}", b);
    let c = m.clone(); println!("{} {:?}", c == m, c.into_iter().collect::<Vec<_>>());
    println!("{:?}", m.iter().map(|(k, _)| k).collect::<Vec<_>>());
}
