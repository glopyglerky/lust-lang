// expect: L0306
// rustc E0597: the map's &str key borrows s, which drops first.
use std::collections::HashMap; fn main() { let mut m: HashMap<&str, u32> = HashMap::new(); { let s = String::from("k"); m.insert(&s, 1); } let v: Vec<_> = m.drain().collect(); println!("{:?}", v); }
