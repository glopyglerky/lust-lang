// expect: L0110
// rustc E0308: `HashMap::clone` takes a `&HashMap`.
use std::collections::HashMap;
fn main() { let v = vec![1u8]; let _m: HashMap<u8, u8> = HashMap::clone(&v); }
