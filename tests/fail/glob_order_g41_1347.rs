// expect: L0509
// fx-glob2 sweep case g41_1347. `use crate::h0::x` reads an ambiguity that rustc carries on through `h0`; rustc reports E0659.
// The answer depends on rustc's glob event order; lust rejects the class (subset rule).
mod h0 { pub(crate) use self::c::*; pub mod c { pub fn cf() -> u32 { 5 } pub use crate::i1::*; use crate::i3::*; } }
mod i2 { pub struct X {} impl X { pub fn v() -> u32 { 2 } } pub fn x() -> u32 { 20 } }
mod h1 { pub use crate::h0::*; use crate::i3::x; pub use crate::i3::*; pub(crate) use crate::i2::*; }
mod i3 { pub struct X {} impl X { pub fn v() -> u32 { 3 } } pub fn x() -> u32 { 30 } }
mod i1 { pub struct X {} impl X { pub fn v() -> u32 { 1 } } pub fn x() -> u32 { 10 } }
mod h2 { pub mod c { pub use crate::i3::*; pub fn cf() -> u32 { 5 } pub use crate::h1::*; } pub(crate) use crate::h0::x; pub use self::c::*; }
fn main() { println!("{}", crate::h2::c::cf()); }


