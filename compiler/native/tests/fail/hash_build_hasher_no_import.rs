// expect: L0104
// `build_hasher` is a BuildHasher method: without the trait in scope
// RandomState has no such method (rustc: E0599).
use std::hash::{Hasher, RandomState};

fn main() {
    let s = RandomState::new();
    let mut h = s.build_hasher();
    h.write_u8(1);
    println!("{}", h.finish() > 0);
}
