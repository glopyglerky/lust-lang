// `HashMap::with_capacity(usize::MAX)` panics with hashbrown's capacity
// overflow; under panic=unwind the table is left empty and nothing
// allocates after the panic (it used to go on to an allocation failure).
use std::collections::{HashMap, HashSet};

fn main() {
    let r = std::panic::catch_unwind(|| HashMap::<u64, char>::with_capacity(usize::MAX).len());
    println!("map caught={}", r.is_err());
    let r = std::panic::catch_unwind(|| HashSet::<u64>::with_capacity(usize::MAX).len());
    println!("set caught={}", r.is_err());
    let r = std::panic::catch_unwind(|| {
        let mut m: HashMap<u8, u8> = HashMap::new();
        m.insert(1, 2);
        m.reserve(usize::MAX);
        m.len()
    });
    println!("reserve caught={}", r.is_err());
}
