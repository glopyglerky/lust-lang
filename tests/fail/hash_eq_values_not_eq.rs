// expect: L0112
// rustc E0369: the values are not PartialEq.
use std::collections::HashMap;
struct N;
fn main() {
    let a: HashMap<u8, N> = HashMap::new();
    let b: HashMap<u8, N> = HashMap::new();
    println!("{}", a == b);
}
