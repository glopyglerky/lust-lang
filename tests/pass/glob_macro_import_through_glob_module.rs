// `q` is in scope only through the block glob, for a macro import too.
mod m { pub mod q { macro_rules! z { () => { 5u32 } } pub(crate) use z; } }
fn main() { { use m::*; use q::z; println!("{}", z!()); } }
