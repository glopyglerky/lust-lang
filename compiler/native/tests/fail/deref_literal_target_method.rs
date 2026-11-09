// expect: L0111
// rustc E0689: a method on `*m` whose type is still an open integer literal.
use std::mem::ManuallyDrop;
fn main() { let m = ManuallyDrop::new(5); println!("{}", (*m).pow(2)); }
