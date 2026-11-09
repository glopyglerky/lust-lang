// expect: L0306
// rustc E0597: the map stores a borrow of `s`, which dies first.
use std::collections::HashMap;

fn main() {
    let mut m: HashMap<u32, &str> = HashMap::new();
    {
        let s = String::from("x");
        m.insert(1, &s);
    }
    println!("{:?}", m);
}
