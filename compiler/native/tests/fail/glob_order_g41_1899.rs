// expect: L0509
// fx-glob2 sweep case g41_1899. `h0::x` comes through globs of different visibility; rustc keeps a private one (E0603).
// The answer depends on rustc's glob event order; lust rejects the class (subset rule).
mod i3 { pub struct X {} impl X { pub fn v() -> u32 { 3 } } pub fn x() -> u32 { 30 } }
mod h1 { use crate::i3::*; pub(crate) use crate::h0::*; }
mod h0 { use self::c::*; pub mod c { pub(super) use crate::i3::*; pub(crate) use crate::h1::*; pub fn cf() -> u32 { 5 } } pub use crate::i3::*; }
mod h2 { use self::c::*; pub(crate) use crate::i3::X; pub(crate) use crate::i3::*; pub mod c { pub fn cf() -> u32 { 5 } pub(crate) use crate::i2::*; use crate::i3::*; } use crate::i1::*; pub use crate::i3::x; }
mod u6 { use crate::h0::*; pub fn f() -> u32 { X::v() } }
mod i1 { pub struct X {} impl X { pub fn v() -> u32 { 1 } } pub fn x() -> u32 { 10 } }
mod i2 { pub struct X {} impl X { pub fn v() -> u32 { 2 } } pub fn x() -> u32 { 20 } }
fn main() { println!("{}", crate::u6::f()); println!("{}", crate::h0::c::cf()); println!("{}", crate::h0::x()); }


