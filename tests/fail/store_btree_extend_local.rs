// expect: L0306
// rustc E0597: `BTreeMap::extend` stores the pairs' borrows in the map.
use std::collections::BTreeMap;
fn main() {
    let mut m = BTreeMap::new();
    {
        let s = 5u8;
        m.extend([(1u8, &s)]);
    }
    println!("{}", m.len());
}
