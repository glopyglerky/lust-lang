// expect: L0308
// rustc E0596: `m[&k].push(..)` needs IndexMut.
use std::collections::HashMap;
fn main() {
    let mut m: HashMap<u8, Vec<u8>> = HashMap::new();
    m.insert(1, vec![]);
    m[&1].push(4);
    println!("{:?}", m[&1]);
}
