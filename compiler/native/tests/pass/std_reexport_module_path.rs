// A path through a crate module to a std item that module re-exports.
mod m {
    pub use std::cmp::max;
    pub use std::collections::HashMap;
    pub use std::mem;
    pub use std::mem::swap as exchange;
    pub use std::thread::spawn;
    pub mod n {
        pub use std::panic::*;
        pub fn twice(x: i32) -> i32 {
            super::max(x, 0) * 2
        }
    }
}

fn main() {
    let h = m::spawn(|| 5);
    let mut a = 1;
    let mut b = 2;
    m::exchange(&mut a, &mut b);
    m::mem::swap(&mut a, &mut b);
    crate::m::mem::swap(&mut a, &mut b);
    let mut map: m::HashMap<u8, u8> = m::HashMap::new();
    map.insert(1, 2);
    let r = m::n::catch_unwind(m::n::AssertUnwindSafe(|| 3));
    println!("{} {} {} {} {:?} {}", h.join().unwrap(), a, b, map.len(), r.ok(), m::n::twice(4));
}
