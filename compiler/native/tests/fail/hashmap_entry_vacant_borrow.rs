// expect: L0306
// rustc E0597: the vacant entry stores a reference to `x`, which dies first.
use std::collections::HashMap;
use std::collections::hash_map::Entry;
fn main() {
    let mut m: HashMap<u32, &u32> = HashMap::new();
    {
        let x = 5;
        if let Entry::Vacant(v) = m.entry(1) {
            v.insert(&x);
        }
    }
    println!("{}", m.len());
}
