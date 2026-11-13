// expect: L0507
// rustc E0659: a module-level `use` an expansion made binds in the whole
// module, so the earlier `vec!` call is ambiguous with the std macro.
mod a { macro_rules! mine { ($($t:tt)*) => { 7 } } pub(crate) use mine; }
fn main() { let v = vec![1]; println!("{:?}", v); }
macro_rules! d { () => { use crate::a::mine as vec; } }
d!();
