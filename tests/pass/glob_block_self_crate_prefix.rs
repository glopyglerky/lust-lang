// A `crate::` first segment cannot be shadowed by the glob.
mod a { pub mod a { pub fn f() -> u32 { 11 } } pub fn x() -> u32 { 1 } }
fn main() { { use crate::a::*; println!("{}", x()); } }
