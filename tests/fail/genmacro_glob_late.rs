// expect: L0507
// rustc E0659: a block glob binds in its whole block, and a glob-brought
// `vec` is ambiguous with the std macro.
mod a { macro_rules! mine { ($($t:tt)*) => { 7 } } pub(crate) use mine as vec; }
fn main() { let v = vec![1]; println!("{:?}", v); use crate::a::*; }
