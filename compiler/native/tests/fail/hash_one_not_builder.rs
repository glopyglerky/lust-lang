// expect: L0104
// rustc E0599: `u32` is not a `BuildHasher`.
#[allow(unused_imports)]
use std::hash::BuildHasher;
fn main() { let x = 3u32; println!("{}", x.hash_one(1u8)); }
