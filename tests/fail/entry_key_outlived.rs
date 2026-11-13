// expect: L0306
// rustc E0597: the probe key is stored in the map.
use std::collections::HashMap;

fn main() {
    let mut m: HashMap<&str, u8> = HashMap::new();
    {
        let s = String::from("k");
        m.entry(&s).or_insert(1);
    }
    println!("{m:?}");
}
