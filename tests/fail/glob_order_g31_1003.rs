// expect: L0507
// fx-glob2 sweep case g31_1003. The same item reaches `h3` and `h2` through globs of different visibility; rustc finds `X` ambiguous in `u8` (E0659).
// lust follows rustc's widest glob visibility, so it finds the ambiguity too.
mod i1 { pub struct X {} impl X { pub fn v() -> u32 { 1 } } pub fn x() -> u32 { 10 } }
mod h2 { pub(crate) use crate::i3::x; pub(crate) use crate::i1::*; use crate::i3::X; pub use crate::i3::*; }
mod h1 { pub use crate::h3::*; pub(crate) use crate::h2::*; pub mod c { pub(crate) use crate::h0::*; pub fn cf() -> u32 { 5 } use crate::i1::*; } pub use self::c::*; }
mod h3 { pub struct X {} impl X { pub fn v() -> u32 { 103 } } pub(crate) use crate::h2::*; }
mod h0 { pub use crate::i1::*; pub(crate) use crate::h3::*; pub use crate::i1::x; pub(crate) use crate::i1::X; pub use crate::i3::*; }
mod u7 { use crate::h2::*; pub fn f() -> u32 { x() } }
mod i2 { pub struct X {} impl X { pub fn v() -> u32 { 2 } } pub fn x() -> u32 { 20 } }
mod i3 { pub struct X {} impl X { pub fn v() -> u32 { 3 } } pub fn x() -> u32 { 30 } }
mod u8 { use crate::h2::*; use crate::h3::*; pub fn f() -> u32 { X::v() + x() } }
fn main() { println!("{}", crate::u7::f()); println!("{}", crate::h1::X::v()); println!("{}", crate::u8::f()); }


