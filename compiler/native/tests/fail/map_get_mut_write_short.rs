// expect: L0306
// rustc E0597: `get_mut` gives `&mut V`, so a short borrow written there must outlive the map.
use std::collections::HashMap;
static A: u8 = 1;
fn main() {
    let mut m: HashMap<u32, &u8> = HashMap::new();
    m.insert(1, &A);
    {
        let b = 5u8;
        if let Some(x) = m.get_mut(&1) {
            *x = &b;
        }
    }
    println!("{}", m.len());
}
