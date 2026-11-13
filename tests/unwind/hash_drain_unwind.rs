// A panic while a HashMap drain is iterated (R20): the drain drops during
// the unwind, which drops the entries not yet taken in table order and
// gives the emptied table (capacity kept) back to the map.
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, DefaultHasher};
struct D(u32);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
fn main() {
    let mut m: HashMap<u32, D, BuildHasherDefault<DefaultHasher>> = HashMap::default();
    for i in 0..5 {
        m.insert(i, D(i));
    }
    let cap = m.capacity();
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        for (k, v) in m.drain() {
            if k == 2 {
                panic!("boom");
            }
            println!("got {}", v.0);
        }
    }));
    println!("{} {} {}", r.is_err(), m.len(), m.capacity() == cap);
    m.insert(9, D(9));
}
