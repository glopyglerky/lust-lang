// expect: L0507
// rustc E0659: a glob-brought macro named `assert` is ambiguous with
// the std macro of that name.
mod b { macro_rules! assert { () => { 3u32 } } pub(crate) use assert; }
use b::*;
fn main() { println!("{}", assert!()); }
