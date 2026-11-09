// expect: L0509
// fx-glob2 sweep case g53_0885. `h2::X` comes through globs of different visibility; rustc keeps a private one (E0603).
// The answer depends on rustc's glob event order; lust rejects the class (subset rule).
mod h3 { pub use crate::h1::*; pub use crate::h2::*; pub(crate) use crate::i3::*; }
mod h2 { pub(crate) use self::c::*; pub mod c { pub fn cf() -> u32 { 5 } pub(super) use crate::h1::*; pub(crate) use crate::i3::*; } pub use crate::h0::*; }
mod i1 { pub struct X {} impl X { pub fn v() -> u32 { 1 } } pub fn x() -> u32 { 10 } }
mod h0 { pub use self::c::*; pub mod c { pub use crate::i3::*; pub fn cf() -> u32 { 5 } pub use crate::h2::*; pub mod g { use super::*; pub fn gv() -> u32 { X::v() } } } pub(crate) use crate::i3::*; }
mod i2 { pub struct X {} impl X { pub fn v() -> u32 { 2 } } pub fn x() -> u32 { 20 } }
mod h1 { pub use crate::h3::*; }
mod i3 { pub struct X {} impl X { pub fn v() -> u32 { 3 } } pub fn x() -> u32 { 30 } }
fn main() { println!("{}", crate::h3::X::v()); println!("{}", crate::h2::X::v()); println!("{}", crate::h1::X::v()); }


