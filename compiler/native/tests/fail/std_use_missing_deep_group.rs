// expect: L0506
// A missing item next to a real one in a std group (rustc E0432).
use std::collections::{HashMap, Nope};

fn main() {
    let m: HashMap<u8, u8> = HashMap::new();
    println!("{}", m.len());
}
