// expect: L0308
// rustc E0596: drain needs a mutable map.
use std::collections::HashMap; fn main() { let m: HashMap<u32, u32> = HashMap::new(); let n = m.drain().count(); println!("{}", n); }
