use std::collections::BTreeMap;
fn main() {
    let mut m: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    println!("{:?}", m.insert(String::from("bx"), vec![3]));
    let c = m.clone(); println!("{} {:?}", c == m, c.into_iter().collect::<Vec<_>>());
    println!("{:?} {}", m, m.len());
    println!("{:?} {:?}", m.first_key_value(), m.last_key_value());
    println!("{:#?} {}", m, m.len());
    let c = m.clone(); println!("{} {:?}", c == m, c.into_iter().collect::<Vec<_>>());
    println!("{:?} {:?}", m.get(&String::from("dyy")), m.contains_key(&String::from("f")));
}
