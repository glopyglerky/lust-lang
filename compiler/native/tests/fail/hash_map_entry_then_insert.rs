// expect: L0303
// rustc E0499: the Entry holds the map mutably borrowed.
use std::collections::HashMap;

fn main() {
    let mut m: HashMap<u32, u32> = HashMap::new();
    let e = m.entry(1);
    m.insert(2, 2);
    e.or_insert(3);
    println!("{:?}", m);
}
