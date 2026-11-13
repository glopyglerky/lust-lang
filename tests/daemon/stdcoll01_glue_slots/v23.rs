use std::collections::BTreeMap;
fn main() {
    let mut m: BTreeMap<(u8, bool), Vec<u8>> = BTreeMap::new();
    println!("{:?}", m.remove(&(0, true)));
    println!("{:?} {:?}", m.get(&(0, true)), m.contains_key(&(1, true)));
    let hi = m.split_off(&(3, true)); println!("{:?} {:?}", m, hi); let mut hi = hi; m.append(&mut hi);
    let hi = m.split_off(&(0, false)); println!("{:?} {:?}", m, hi); let mut hi = hi; m.append(&mut hi);
    let c = m.clone(); println!("{} {:?}", c == m, c.into_iter().collect::<Vec<_>>());
    let c = m.clone(); println!("{} {:?}", c == m, c.into_iter().collect::<Vec<_>>());
    let mut i = 0; m.retain(|_, _| { i += 1; i % 2 == 0 }); println!("{:?}", m);
    m.entry((3, false)).or_insert(vec![2]); println!("{:?}", m);
}
