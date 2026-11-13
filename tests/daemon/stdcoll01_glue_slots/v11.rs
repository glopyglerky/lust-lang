use std::collections::BTreeMap;
fn main() {
    let mut m: BTreeMap<u8, i64> = BTreeMap::new();
    println!("{} {:?}", m.keys().count(), m.values().rev().take(2).collect::<Vec<_>>());
    println!("{:?}", m.pop_first());
    println!("{:?}", m.iter().map(|(k, _)| k).collect::<Vec<_>>());
    println!("{:?}", m.pop_first());
    println!("{:?}", m.insert(9, -63));
    println!("{:?}", m.insert(8, -33));
    println!("{} {:?}", m.keys().count(), m.values().rev().take(2).collect::<Vec<_>>());
    println!("{:?}", m.iter().map(|(k, _)| k).collect::<Vec<_>>());
    println!("{:?} {}", m, m.len());
    println!("{:?}", m.insert(11, -50));
    println!("{:?} {}", m, m.len());
    println!("{:?}", m.insert(8, -53));
}
