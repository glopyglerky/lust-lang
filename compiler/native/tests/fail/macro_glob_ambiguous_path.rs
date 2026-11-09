// expect: L0802
// A path call of a macro two globs make ambiguous finds nothing (rustc E0659).
mod m1 { macro_rules! k { () => { 1u32 }; } pub(crate) use k; }
mod m2 { macro_rules! k { () => { 2u32 }; } pub(crate) use k; }
mod prelude { pub(crate) use crate::m1::*; pub(crate) use crate::m2::*; }
mod p2 { pub(crate) use crate::prelude::*; }
fn main() { println!("{}", crate::p2::k!()); }
