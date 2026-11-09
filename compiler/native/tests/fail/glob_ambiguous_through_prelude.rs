// expect: L0507
mod prelude {
    pub use crate::ia::*;
    pub use crate::ib::*;
}
mod ia { pub fn who() -> u32 { 1 } }
mod ib { pub fn who() -> u32 { 2 } }
mod user {
    use crate::prelude::*;
    pub fn get() -> u32 { who() }
}
fn main() { println!("{}", user::get()); }
