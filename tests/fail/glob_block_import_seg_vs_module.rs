// expect: L0507
// rustc E0659: an import path in the block resolves `q` under import
// resolution, where the glob-brought `m::q` does not shadow the module `q`.
mod m { pub mod q { pub fn g() -> u32 { 1 } } }
mod q { pub fn g() -> u32 { 9 } }
fn main() { { use m::*; use q::g; println!("{}", g()); } }
