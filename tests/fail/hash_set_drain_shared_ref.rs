// expect: L0308
// rustc E0596: drain through a shared reference.
use std::collections::HashSet; fn main() { let s: HashSet<u8> = [1u8, 2].into_iter().collect(); let r = &s; let n = r.drain().count(); println!("{}", n); }
