// expect: L0303
// rustc E0502: the value reference keeps the map borrowed.
use std::collections::HashMap;

fn main() {
    let mut m: HashMap<u8, u8> = HashMap::new();
    let r = m.entry(1).or_insert(0);
    let n = m.len();
    *r += 1;
    println!("{n}");
}
