// expect: L0104
// rustc E0599: insert_entry returns an OccupiedEntry, which has no or_insert.
use std::collections::HashMap;
fn main() {
    let mut m: HashMap<u8, u8> = HashMap::new();
    let r = m.entry(1).insert_entry(2).or_insert(3);
    println!("{r}");
}
