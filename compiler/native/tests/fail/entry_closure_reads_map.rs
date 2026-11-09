// expect: L0303
// rustc E0502: the closure reads the map the Entry borrows.
use std::collections::HashMap;

fn main() {
    let mut m: HashMap<u8, usize> = HashMap::new();
    m.entry(1).or_insert_with(|| m.len());
    println!("{m:?}");
}
