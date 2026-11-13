// expect: L0733
// rustc: lifetime may not live long enough. The value slot is `'static`.
use std::collections::HashMap;
fn put<'a>(m: &mut HashMap<&'a str, &'static str>, k: &'a str) {
    m.insert(k, k);
}
fn main() {
    let s = String::from("x");
    let mut m = HashMap::new();
    put(&mut m, &s);
    println!("{}", m.len());
}
