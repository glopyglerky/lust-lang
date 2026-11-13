// expect: L0303
// rustc E0502: insert while the loop borrows the map.
use std::collections::HashMap;

fn main() {
    let mut m: HashMap<u32, u32> = HashMap::new();
    m.insert(1, 1);
    for (k, _) in &m {
        m.insert(*k + 1, 0);
    }
    println!("{}", m.len());
}
