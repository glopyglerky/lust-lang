// expect: L0509
// fx-glob2 sweep case g41_0376. `h0` gets `X` through globs of different visibility; rustc keeps a private one (E0603).
// The answer depends on rustc's glob event order; lust rejects the class (subset rule).
mod i1 { pub struct X {} impl X { pub fn v() -> u32 { 1 } } pub fn x() -> u32 { 10 } }
mod i2 { pub struct X {} impl X { pub fn v() -> u32 { 2 } } pub fn x() -> u32 { 20 } }
mod u7 { use crate::h1::*; use crate::h0::*; pub fn f() -> u32 { X::v() } }
mod h3 { pub(crate) use crate::i2::*; }
mod h0 { pub mod c { pub(super) use crate::h3::*; pub fn cf() -> u32 { 5 } pub use crate::h2::*; } pub(crate) use crate::h3::*; use self::c::*; }
mod i3 { pub struct X {} impl X { pub fn v() -> u32 { 3 } } pub fn x() -> u32 { 30 } }
mod h1 { pub use crate::h3::*; pub use crate::h2::*; use crate::i2::*; }
mod h2 { use crate::h1::*; pub use crate::i3::x; pub use crate::i2::*; pub(crate) use crate::i3::*; pub(crate) use crate::i2::X; }
fn main() { println!("{}", crate::h0::X::v()); println!("{}", crate::u7::f()); println!("{}", crate::h3::X::v()); }


