// expect: L0303
// rustc E0502: the `&V` item from `values()` keeps the map borrowed.
use std::collections::HashMap;
fn main() {
    let mut m: HashMap<u32, String> = HashMap::new();
    m.insert(1, String::from("a"));
    let x = m.values().next();
    m.insert(2, String::from("b"));
    println!("{:?}", x);
}
