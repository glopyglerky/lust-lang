// expect: L0303
// rustc E0502: the index borrows the map while the reference lives.
use std::collections::HashMap;
fn main() {
    let mut m: HashMap<u8, String> = HashMap::new();
    m.insert(1, String::from("a"));
    let r = &m[&1];
    m.insert(2, String::from("b"));
    println!("{}", r);
}
