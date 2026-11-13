// An explicit import in the outer block shadows the module for an
// import in the inner block (prints the explicit target).
mod m { pub mod q { pub fn g() -> u32 { 1 } } }
mod q { pub fn g() -> u32 { 9 } }
fn main() { use m::q; { use q::g; println!("{}", g()); } }
