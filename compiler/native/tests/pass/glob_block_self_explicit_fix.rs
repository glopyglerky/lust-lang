// An explicit import of `a` in the same block settles the glob
// path's first segment, so `use a::*` is fine.
mod a { pub mod a { pub fn f() -> u32 { 11 } } pub fn x() -> u32 { 1 } }
fn main() { { use crate::a; use a::*; println!("{}", x()); } }
