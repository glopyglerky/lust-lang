// Only the glob names `q` here, so an import through it is fine.
mod m { pub mod q { pub fn g() -> u32 { 1 } } }
fn main() { { use m::*; use q::g; println!("{}", g()); } }
