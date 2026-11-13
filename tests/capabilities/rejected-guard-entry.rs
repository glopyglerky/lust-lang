// capability: rejected: match guards over a bound map Entry payload
// expect: L9001

use std::collections::HashMap;
use std::collections::hash_map::Entry;

fn main() {
    let mut m: HashMap<u32, u32> = HashMap::new();
    match m.entry(1) {
        Entry::Occupied(o) if *o.get() > 0 => println!("{}", o.get()),
        _ => println!("none"),
    }
}
