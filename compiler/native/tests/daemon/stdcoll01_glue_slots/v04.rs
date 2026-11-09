use std::collections::BTreeMap;
fn main() {
    let mut m: BTreeMap<String, &str> = BTreeMap::new();
    println!("{:?} {:?}", m.first_key_value(), m.last_key_value());
    println!("{:#?} {}", m, m.len());
    println!("{:?}", m.pop_first());
    println!("{:?}", m.pop_first());
    let b: BTreeMap<String, &str> = BTreeMap::from_iter(vec![(String::from("ax"), "w"), (String::from("dx"), "w"), (String::from("fx"), "")].into_iter()); println!("{:?}", b);
    let mut i = 0; m.retain(|_, _| { i += 1; i % 2 == 0 }); println!("{:?}", m);
    println!("{:?}", m.insert(String::from("ayy"), "v"));
    let b: BTreeMap<String, &str> = BTreeMap::from_iter(vec![(String::from("d"), ""), (String::from("cx"), "w"), (String::from("gyy"), "v")].into_iter()); println!("{:?}", b);
    println!("{:?} {:?}", m.get(&String::from("dx")), m.contains_key(&String::from("cyy")));
    println!("{:?}", m.insert(String::from("ex"), ""));
}
