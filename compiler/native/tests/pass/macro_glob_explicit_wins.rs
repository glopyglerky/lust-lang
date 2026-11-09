// Two globs bring different macros named `k`; an explicit import of one wins.
mod m1 { macro_rules! k { () => { 1u32 }; } pub(crate) use k; }
mod m2 { macro_rules! k { () => { 2u32 }; } pub(crate) use k; }
mod prelude { pub(crate) use crate::m1::*; pub(crate) use crate::m2::*; pub(crate) use crate::m2::k; }
mod user { use crate::prelude::*; pub fn f() -> u32 { k!() } }
mod other { use crate::m1::*; use crate::m2::*; use crate::m1::k; pub fn g() -> u32 { k!() } }
fn main() { println!("{} {}", user::f(), other::g()); }
