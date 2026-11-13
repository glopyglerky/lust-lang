// expect: L0306
// rustc E0597: values_mut stores a borrow of `s` into the map.
use std::collections::HashMap;

fn main() {
    let mut m: HashMap<u32, &str> = HashMap::new();
    m.insert(1, "a");
    {
        let s = String::from("x");
        for v in m.values_mut() {
            *v = &s;
        }
    }
    println!("{:?}", m);
}
