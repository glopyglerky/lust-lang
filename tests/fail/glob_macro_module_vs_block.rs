// expect: L0507
// rustc E0659: the block glob macro is ambiguous with the module
// glob macro (an ordinary name would shadow here).
mod a { macro_rules! k { () => { 1u32 } } pub(crate) use k; }
mod b { macro_rules! k { () => { 2u32 } } pub(crate) use k; }
use a::*;
fn main() { { use b::*; println!("{}", k!()); } }
