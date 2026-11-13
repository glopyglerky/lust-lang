// expect: L0303
// rustc E0502: the Drain holds the map's &mut loan while it lives.
use std::collections::HashMap; fn main() { let mut m: HashMap<u32, u32> = HashMap::new(); m.insert(1, 1); let d = m.drain(); println!("{}", m.len()); drop(d); }
