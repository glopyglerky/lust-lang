use std::collections::BTreeMap;
fn main() {
    let mut m: BTreeMap<i32, i64> = BTreeMap::new();
    println!("{:?} {}", m, m.len());
    println!("{:?} {:?}", m.get(&14), m.contains_key(&14));
    println!("{:?}", m.insert(5, 16));
    println!("{:?}", m.remove(&-1));
    let hi = m.split_off(&11); println!("{:?} {:?}", m, hi); let mut hi = hi; m.append(&mut hi);
    println!("{:?}", m.remove(&2));
    println!("{:#?} {}", m, m.len());
    println!("{:?}", m.insert(-4, -48));
    println!("{:?}", m.insert(9, -67));
    let hi = m.split_off(&12); println!("{:?} {:?}", m, hi); let mut hi = hi; m.append(&mut hi);
}
