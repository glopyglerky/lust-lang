// expect: L0507
// rustc E0659: the first segment of a macro call path resolves under
// macro resolution, so the glob-brought `a::q` does not shadow `q`.
mod a { pub mod q { macro_rules! z { () => { 1u32 } } pub(crate) use z; } }
mod q { macro_rules! z { () => { 9u32 } } pub(crate) use z; }
fn main() { { use a::*; println!("{}", q::z!()); } }
