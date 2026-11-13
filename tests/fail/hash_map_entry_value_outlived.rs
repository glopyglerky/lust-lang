// expect: L0306
// rustc E0597: or_insert stores a borrow of `s` through the Entry.
use std::collections::HashMap;

fn main() {
    let mut m: HashMap<u32, &str> = HashMap::new();
    {
        let s = String::from("x");
        m.entry(1).or_insert(&s);
    }
    println!("{:?}", m);
}
