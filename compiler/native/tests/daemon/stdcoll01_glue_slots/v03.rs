use std::collections::BTreeMap;
fn main() {
    let mut m: BTreeMap<String, i64> = BTreeMap::new();
    println!("{:?}", m.iter().map(|(k, _)| k).collect::<Vec<_>>());
    println!("{:?}", m.pop_first());
    println!("{:?}", m.pop_first());
    println!("{:?} {:?}", m.get(&String::from("bx")), m.contains_key(&String::from("ex")));
    println!("{:?}", m.insert(String::from("cyy"), -5));
    println!("{:?} {}", m, m.len());
    println!("{:?} {}", m, m.len());
    let hi = m.split_off(&String::from("f")); println!("{:?} {:?}", m, hi); let mut hi = hi; m.append(&mut hi);
    println!("{:?} {:?}", m.get(&String::from("dx")), m.contains_key(&String::from("f")));
    println!("{:?}", m.iter().map(|(k, _)| k).collect::<Vec<_>>());
    let mut i = 0; m.retain(|_, _| { i += 1; i % 2 == 0 }); println!("{:?}", m);
    println!("{:?}", m.insert(String::from("cx"), 44));
    let b: BTreeMap<String, i64> = BTreeMap::from_iter(vec![(String::from("f"), -15), (String::from("fyy"), -34), (String::from("bx"), -44)].into_iter()); println!("{:?}", b);
    let c = m.clone(); println!("{} {:?}", c == m, c.into_iter().collect::<Vec<_>>());
    println!("{:?} {:?}", m.first_key_value(), m.last_key_value());
}
