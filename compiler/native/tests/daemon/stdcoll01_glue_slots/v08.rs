use std::collections::BTreeMap;
fn main() {
    let mut m: BTreeMap<char, &str> = BTreeMap::new();
    println!("{:?} {:?}", m.first_key_value(), m.last_key_value());
    println!("{:?}", m.insert('t', ""));
    println!("{:?}", m.insert('t', ""));
    println!("{:?}", m.insert('e', ""));
    println!("{:?}", m.insert('w', "v"));
    let hi = m.split_off(&'t'); println!("{:?} {:?}", m, hi); let mut hi = hi; m.append(&mut hi);
    println!("{:?}", m.insert('y', "w"));
    println!("{:#?} {}", m, m.len());
}
