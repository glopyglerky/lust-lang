// panic: no entry found for key
// A missing key panics with std's `expect` text (W-STD-B).
use std::collections::HashMap;
fn main() {
    let m: HashMap<u8, u8> = HashMap::from([(1, 2)]);
    println!("{}", m[&1]);
    println!("{}", m[&9]);
}
