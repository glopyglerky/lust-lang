// expect: L0507
// rustc E0659: the outer block's glob brings `m::q`; an import through
// `q` in the inner block is ambiguous with the module `q`.
mod m { pub mod q { pub fn g() -> u32 { 1 } } }
mod q { pub fn g() -> u32 { 9 } }
fn main() { use m::*; { use q::g; println!("{}", g()); } }
