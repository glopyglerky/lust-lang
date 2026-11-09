// An explicit import of the item one glob brings shadows every glob,
// also when the other glob's chain resolves only on a later sweep.
mod prelude {
    pub use crate::ca0::*;
    pub use crate::ib::*;
    pub use crate::ib::who;
}
mod user {
    use crate::prelude::*;
    pub fn get() -> u32 { who() }
}
mod ca0 { pub use crate::ia::*; }
mod ib { pub fn who() -> u32 { 2 } }
mod ia { pub fn who() -> u32 { 1 } }
fn main() { println!("{}", user::get()); }
