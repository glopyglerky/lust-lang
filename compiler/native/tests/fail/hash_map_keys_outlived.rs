// expect: L0306
// rustc E0597: the keys borrow the map, which dies first.
use std::collections::HashMap;

fn main() {
    let k: Vec<&u32>;
    {
        let mut m: HashMap<u32, u32> = HashMap::new();
        m.insert(1, 1);
        k = m.keys().collect();
    }
    println!("{:?}", k);
}
