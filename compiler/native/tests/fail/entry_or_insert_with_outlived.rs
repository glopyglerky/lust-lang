// expect: L0306
// rustc E0597: the value or_insert_with stores borrows a dropped String.
use std::collections::HashMap;

fn main() {
    let mut m: HashMap<u8, &str> = HashMap::new();
    {
        let s = String::from("x");
        m.entry(1).or_insert_with(|| &s);
    }
    println!("{m:?}");
}
