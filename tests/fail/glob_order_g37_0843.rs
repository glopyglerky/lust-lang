// expect: L0507
// fx-glob2 sweep case g37_0843. A `use super::*` grandchild reads `X`, which a glob cycle carries back with another visibility; rustc finds it ambiguous (E0659).
// lust follows rustc's widest glob visibility, so it finds the ambiguity too.
mod i2 { pub struct X {} impl X { pub fn v() -> u32 { 2 } } pub fn x() -> u32 { 20 } }
mod h2 { pub use crate::h3::*; pub use crate::i3::*; pub(crate) use crate::h0::*; pub(crate) use self::c::*; pub use crate::i2::x; use crate::i3::X; pub mod c { pub use crate::i3::*; pub mod g { use super::*; pub fn gv() -> u32 { X::v() } } pub fn cf() -> u32 { 5 } pub(super) use crate::h3::*; } }
mod i1 { pub struct X {} impl X { pub fn v() -> u32 { 1 } } pub fn x() -> u32 { 10 } }
mod h0 { pub(crate) use crate::h2::*; pub use crate::i2::*; pub(crate) use crate::i2::X; pub use crate::i1::*; use crate::i1::x; }
mod i3 { pub struct X {} impl X { pub fn v() -> u32 { 3 } } pub fn x() -> u32 { 30 } }
mod h3 { use crate::i2::*; pub use crate::h0::*; }
mod h1 { pub mod c { pub(crate) use crate::i1::*; pub fn cf() -> u32 { 5 } pub(crate) use crate::h3::*; } pub use self::c::*; pub(crate) use crate::i2::*; }
fn main() { println!("{}", crate::h0::X::v()); }


