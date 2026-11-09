// expect: L0110
// rustc E0308: the alias names std::cmp::Ordering, not the user type.
#[derive(Debug)]
struct Ordering(u8);
use std::cmp::Ordering as O;
fn main() { let x: O = Ordering(1); println!("{:?}", x); }
