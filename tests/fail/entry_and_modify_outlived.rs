// expect: L0306
// rustc E0597: and_modify stores a borrow of a dropped String.
use std::collections::HashMap;

fn main() {
    let mut m: HashMap<u8, &str> = HashMap::new();
    m.insert(1, "a");
    {
        let s = String::from("x");
        m.entry(1).and_modify(|v| *v = &s);
    }
    println!("{m:?}");
}
