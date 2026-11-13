// expect: L0507
// rustc E0659: `use alloc::*` in a block brings the module `alloc::alloc`,
// ambiguous with the extern crate for the glob path itself.
extern crate alloc;
fn main() { { use alloc::*; } println!("ok"); }
