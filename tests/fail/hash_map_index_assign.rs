// expect: L0309
// rustc E0594: HashMap has no IndexMut, so `m[&k] = v` cannot assign.
use std::collections::HashMap;
fn main() {
    let mut m: HashMap<u8, u8> = HashMap::new();
    m.insert(1, 2);
    m[&1] = 5;
    println!("{}", m[&1]);
}
