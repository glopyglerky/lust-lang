use std::collections::BTreeMap;
fn main() {
    let mut m: BTreeMap<(u8, bool), i64> = BTreeMap::new();
    let mut i = 0; m.retain(|_, _| { i += 1; i % 2 == 0 }); println!("{:?}", m);
    let hi = m.split_off(&(1, false)); println!("{:?} {:?}", m, hi); let mut hi = hi; m.append(&mut hi);
    println!("{:?} {}", m, m.len());
    println!("{} {:?}", m.keys().count(), m.values().rev().take(2).collect::<Vec<_>>());
    println!("{:?}", m.remove(&(3, false)));
    println!("{:?}", m.pop_first());
    println!("{:?}", m.insert((2, false), -13));
    println!("{:?}", m.insert((1, false), -83));
}
