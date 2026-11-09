// expect: L0612
// rustc E0631: RefCell::into_inner takes the RefCell by value, not a reference.
use std::cell::RefCell;
fn main() { let v = vec![RefCell::new(1u8)]; let _x: Vec<u8> = v.iter().map(RefCell::into_inner).collect(); }
