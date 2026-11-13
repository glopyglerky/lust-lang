// A module glob and a block glob bring the same macro: no ambiguity.
mod a { macro_rules! k { () => { 1u32 } } pub(crate) use k; }
use a::*;
fn main() { { use a::*; println!("{}", k!()); } }
