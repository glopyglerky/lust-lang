// expect: L0306
// rustc E0597: a write through OccupiedEntry::get_mut stores `&s` in the map, which outlives `s`.
use std::collections::HashMap;
fn main() {
    let mut m: HashMap<u32, &str> = HashMap::new();
    let mut o = m.entry(1).insert_entry("a");
    {
        let s = String::from("x");
        *o.get_mut() = &s;
    }
    println!("{:?}", m);
}
