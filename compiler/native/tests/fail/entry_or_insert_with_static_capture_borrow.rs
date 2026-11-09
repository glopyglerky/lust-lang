// expect: L0306
// rustc E0597: the closure returns a borrow of `s` into a map of `'static` values.
use std::collections::HashMap;
fn main() {
    let mut m: HashMap<u8, &'static str> = HashMap::new();
    let s = String::from("x");
    m.entry(1).or_insert_with(|| s.as_str());
    println!("{:?}", m);
}
