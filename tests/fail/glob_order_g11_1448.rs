// expect: L0509
// fx-glob2 sweep case g11_1448. `pub use crate::h0::x` reads `x`, which globs bring with an order-dependent visibility; rustc reports E0364 and E0659.
// The answer depends on rustc's glob event order; lust rejects the class (subset rule).
mod i1 { pub struct X {} impl X { pub fn v() -> u32 { 1 } } pub fn x() -> u32 { 10 } }
mod i2 { pub struct X {} impl X { pub fn v() -> u32 { 2 } } pub fn x() -> u32 { 20 } }
mod h0 { pub use crate::h2::*; pub use crate::h1::X; pub(crate) use crate::h1::*; }
mod i3 { pub struct X {} impl X { pub fn v() -> u32 { 3 } } pub fn x() -> u32 { 30 } }
mod h1 { pub use crate::i1::*; pub use crate::h2::*; pub(crate) use crate::h0::*; pub use crate::h0::x; pub use self::c::*; pub mod c { use crate::i2::*; pub fn cf() -> u32 { 5 } pub(super) use crate::h2::*; } }
mod h2 { pub mod c { pub(crate) use crate::i1::*; pub fn cf() -> u32 { 5 } use crate::i3::*; } pub(crate) use self::c::*; }
fn main() { println!("{}", crate::h1::c::cf()); }


