// expect: L0507
// rustc E0659: macro resolution does not let the inner block's glob
// shadow the outer block's glob macro of the same name.
mod a { macro_rules! k { () => { 1u32 } } pub(crate) use k; }
mod b { macro_rules! k { () => { 2u32 } } pub(crate) use k; }
fn main() { use a::*; { use b::*; println!("{}", k!()); } }
