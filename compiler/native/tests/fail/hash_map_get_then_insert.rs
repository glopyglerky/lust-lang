// expect: L0303
// rustc E0502: the map is borrowed by `r` while insert needs it mutably.
use std::collections::HashMap;

fn main() {
    let mut m: HashMap<u32, String> = HashMap::new();
    m.insert(1, "a".to_string());
    let r = m.get(&1);
    m.insert(2, "b".to_string());
    println!("{:?}", r);
}
