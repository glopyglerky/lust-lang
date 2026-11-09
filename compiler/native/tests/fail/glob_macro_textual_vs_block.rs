// expect: L0507
// rustc E0659: a block glob macro is ambiguous with a textual
// macro_rules! of an outer scope.
macro_rules! k { () => { 9u32 } }
mod b { macro_rules! k { () => { 2u32 } } pub(crate) use k; }
fn main() { { use b::*; println!("{}", k!()); } }
