// expect: L0507
// rustc E0659: the block glob brings a user module `std`; an import
// through `std` is ambiguous with the extern prelude crate.
mod m { pub mod std { pub mod vec { pub struct Vec; } } }
fn main() { { use m::*; use std::vec::Vec; let _ = Vec; } println!("ok"); }
