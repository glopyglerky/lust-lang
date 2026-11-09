// expect: L0509
// fx-glob2 sweep case g37_0262. `use crate::h3::x` reads an ambiguity that a glob carries on; rustc reports E0659 though nothing uses it.
// The answer depends on rustc's glob event order; lust rejects the class (subset rule).
mod i2 { pub struct X {} impl X { pub fn v() -> u32 { 2 } } pub fn x() -> u32 { 20 } }
mod h2 { pub struct X {} impl X { pub fn v() -> u32 { 102 } } use crate::h1::*; pub use crate::h0::*; }
mod i1 { pub struct X {} impl X { pub fn v() -> u32 { 1 } } pub fn x() -> u32 { 10 } }
mod h0 { pub(crate) use crate::i2::*; use crate::i3::*; }
mod i3 { pub struct X {} impl X { pub fn v() -> u32 { 3 } } pub fn x() -> u32 { 30 } }
mod h3 { pub use crate::i1::*; pub use crate::h0::*; }
mod h1 { pub(crate) use crate::h3::*; use crate::h3::x; pub(crate) use crate::i1::*; use crate::i2::*; }
fn main() { println!("{}", crate::h2::X::v()); println!("{}", crate::h2::X::v()); }


