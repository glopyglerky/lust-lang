// expect: L0509
// fx-glob2 sweep case g53_0893. `h0::X` comes through a public glob and a private glob of the same item; rustc keeps the private one (E0603).
// The answer depends on rustc's glob event order; lust rejects the class (subset rule).
mod i1 { pub struct X {} impl X { pub fn v() -> u32 { 1 } } pub fn x() -> u32 { 10 } }
mod i3 { pub struct X {} impl X { pub fn v() -> u32 { 3 } } pub fn x() -> u32 { 30 } }
mod h1 { pub(crate) use crate::i1::*; }
mod u5 { use crate::h1::*; pub fn f() -> u32 { X::v() + x() } }
mod h0 { pub use crate::i1::*; pub mod c { pub(crate) use crate::h1::*; pub use crate::i1::*; pub fn cf() -> u32 { 5 } pub mod g { use super::*; pub fn gv() -> u32 { X::v() } } } use self::c::*; }
mod i2 { pub struct X {} impl X { pub fn v() -> u32 { 2 } } pub fn x() -> u32 { 20 } }
fn main() { println!("{}", crate::h0::X::v()); println!("{}", crate::u5::f()); }


