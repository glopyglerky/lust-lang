// expect: L0306
// rustc E0597: get_mut writes a borrow of `s` into the map.
use std::collections::HashMap;

fn main() {
    let mut m: HashMap<u32, &str> = HashMap::new();
    m.insert(1, "a");
    {
        let s = String::from("x");
        if let Some(v) = m.get_mut(&1) {
            *v = &s;
        }
    }
    println!("{:?}", m);
}
