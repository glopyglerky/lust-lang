// expect: L0507
// fx-glob2 sweep case g11_0004. Two hubs bring `X` and `x` to `u7` through a glob cycle; rustc finds them ambiguous (E0659).
// lust follows rustc's widest glob visibility, so it finds the ambiguity too.
mod h3 { pub(crate) use crate::i3::*; pub mod c { pub(super) use crate::h2::*; pub fn cf() -> u32 { 5 } pub(crate) use crate::h1::*; } pub(crate) use self::c::*; pub use crate::i2::x; use crate::i3::X; }
mod h1 { pub(crate) use crate::h3::*; use crate::h0::*; pub(crate) use crate::i1::X; pub(crate) use crate::i1::x; pub use crate::i1::*; }
mod u7 { use crate::h0::*; use crate::h1::*; pub fn f() -> u32 { X::v() + x() } }
mod h2 { pub(crate) use crate::h0::*; pub(crate) use crate::i2::*; use self::c::*; pub mod c { use crate::i3::*; pub use crate::h0::*; pub fn cf() -> u32 { 5 } } }
mod i3 { pub struct X {} impl X { pub fn v() -> u32 { 3 } } pub fn x() -> u32 { 30 } }
mod h0 { use crate::i2::*; pub(crate) use crate::h2::*; }
mod i1 { pub struct X {} impl X { pub fn v() -> u32 { 1 } } pub fn x() -> u32 { 10 } }
mod i2 { pub struct X {} impl X { pub fn v() -> u32 { 2 } } pub fn x() -> u32 { 20 } }
fn main() { println!("{}", crate::u7::f()); }


