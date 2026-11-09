// expect: L0507
// rustc E0659 in the value namespace: `use q as qq` resolves `q`, which
// the block glob and the crate fn both name.
mod m { pub fn q() -> u32 { 1 } }
fn q() -> u32 { 9 }
fn main() { { use m::*; use q as qq; println!("{}", qq()); } }
