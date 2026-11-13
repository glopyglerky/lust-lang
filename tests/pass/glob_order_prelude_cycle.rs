// The ts-rust scanner_util shape: a prelude re-exports `su`, and `su` globs the prelude
// back next to a public glob of the defining module. rustc accepts it in all 96 orders.
mod gu { pub mod su { pub fn f() -> u32 { 7 } } pub mod core { pub use super::su::*; } }
mod core2 { pub use crate::gu::core::*; }
mod prelude { pub use crate::core2::*; pub use crate::su::*; }
mod su { use crate::prelude::*; pub use crate::gu::su::*; }
mod user { pub fn g() -> u32 { crate::su::f() } }
mod user2 { use crate::su::f; pub fn g() -> u32 { f() } }
fn main() { println!("{} {}", user::g(), user2::g()); }
