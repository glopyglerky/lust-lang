// expect: L0612
// rustc E0277: f64 is not Eq or Hash.
use std::collections::HashMap;

fn main() {
    let mut m = HashMap::new();
    m.insert(1.5f64, 1);
    println!("{}", m.len());
}
