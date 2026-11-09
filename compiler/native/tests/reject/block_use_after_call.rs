// expect: L0809
// Valid Rust: a block `use` binds in its whole block, so `vec!` here is the
// imported macro. lust resolves macro calls in text order and rejects it.
mod a { macro_rules! mine { ($($t:tt)*) => { "mine" } } pub(crate) use mine; }
fn main() { let r = vec!(1); println!("{:?}", r); use crate::a::mine as vec; }
