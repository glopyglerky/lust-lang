// Two block globs bring the same macro by two paths: no ambiguity.
mod m { macro_rules! k { () => { 1u32 } } pub(crate) use k; }
mod a { pub(crate) use crate::m::k; }
mod b { pub(crate) use crate::m::k; }
fn main() { use a::*; { use b::*; println!("{}", k!()); } }
