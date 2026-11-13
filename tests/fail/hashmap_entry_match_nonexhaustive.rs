// expect: L0115
// rustc E0004: `Entry::Vacant(_)` not covered.
use std::collections::HashMap;
use std::collections::hash_map::Entry;
fn main() {
    let mut m: HashMap<u32, u32> = HashMap::new();
    match m.entry(1) {
        Entry::Occupied(e) => println!("{}", e.get()),
    }
}
