// expect: L0306
// rustc E0597: the predicate stores `&s` in a value of the map, and `s` dies first.
use std::collections::HashMap;
fn main() {
    let mut m: HashMap<u32, &str> = HashMap::new();
    m.insert(1, "a");
    {
        let s = String::from("b");
        m.retain(|_, v| {
            *v = &s;
            true
        });
    }
    println!("{:?}", m);
}
