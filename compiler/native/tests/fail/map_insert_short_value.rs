// expect: L0306
// rustc E0597: the map keeps `&b` after `b` is dropped.
use std::collections::HashMap;
fn main() {
    let a = 1u8;
    let mut m = HashMap::new();
    m.insert(1, &a);
    {
        let b = 2u8;
        m.insert(2, &b);
    }
    println!("{}", m.len());
}
