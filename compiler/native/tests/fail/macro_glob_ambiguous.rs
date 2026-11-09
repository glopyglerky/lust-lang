// expect: L0507
// Two globs re-export different macros named `k`: a plain call is ambiguous (rustc E0659).
// The macro re-export fixpoint used to loop forever on this shape (a daemon wedge).
mod m1 { macro_rules! k { () => { 1u32 }; } pub(crate) use k; }
mod m2 { macro_rules! k { () => { 2u32 }; } pub(crate) use k; }
mod prelude { pub(crate) use crate::m1::*; pub(crate) use crate::m2::*; }
mod user { use crate::prelude::*; pub fn f() -> u32 { k!() } }
fn main() { println!("{}", user::f()); }
