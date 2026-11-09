// capability: rejected: a map Entry variant matched through a reference or with this payload pattern
// expect: L9001

use std::collections::HashMap;
use std::collections::hash_map::Entry;

fn main() {
    let mut m: HashMap<u32, u32> = HashMap::new();
    let e = m.entry(1);
    match &e {
        Entry::Occupied(_) => println!("occupied"),
        Entry::Vacant(_) => println!("vacant"),
    }
}
