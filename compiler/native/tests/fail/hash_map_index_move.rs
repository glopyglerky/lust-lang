// expect: L0305
// rustc E0507: a map index is a place behind a shared reference.
use std::collections::HashMap;
fn main() {
    let mut m: HashMap<u8, String> = HashMap::new();
    m.insert(1, String::from("a"));
    let s = m[&1];
    println!("{}", s);
}
