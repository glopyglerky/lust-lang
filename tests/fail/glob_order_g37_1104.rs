// expect: L0509
// fx-glob2 sweep case g37_1104. `use crate::h2::X` reads an ambiguity that a glob carries on; rustc reports E0659 though nothing uses it.
// The answer depends on rustc's glob event order; lust rejects the class (subset rule).
mod h1 { pub use crate::i1::*; use crate::i3::*; pub(crate) use crate::h2::x; }
mod h3 { pub(crate) use crate::i2::*; use crate::h2::X; pub use crate::i3::x; }
mod i3 { pub struct X {} impl X { pub fn v() -> u32 { 3 } } pub fn x() -> u32 { 30 } }
mod i1 { pub struct X {} impl X { pub fn v() -> u32 { 1 } } pub fn x() -> u32 { 10 } }
mod h2 { pub use crate::i1::*; pub use crate::h1::*; use crate::h0::*; }
mod h0 { pub use crate::i1::*; }
mod i2 { pub struct X {} impl X { pub fn v() -> u32 { 2 } } pub fn x() -> u32 { 20 } }
fn main() { println!("{}", crate::h3::X::v()); }


