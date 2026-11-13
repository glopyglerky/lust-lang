// expect: L0507
// rustc E0659: the inner glob brings the same `a::q` the outer block
// imported explicitly, but the module `q` behind that import still
// counts: the outward search does not stop at the same item.
mod a { pub mod q { pub fn g() -> u32 { 1 } } }
mod q { pub fn g() -> u32 { 9 } }
fn main() { use a::q; { use a::*; use q::g; println!("{}", g()); } }
