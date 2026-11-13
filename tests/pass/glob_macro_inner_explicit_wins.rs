// An explicit macro import in the inner block wins over outer globs.
mod a { macro_rules! k { () => { 1u32 } } pub(crate) use k; }
mod b { macro_rules! k { () => { 2u32 } } pub(crate) use k; }
fn main() { use a::*; { use b::k; println!("{}", k!()); } }
