// expect: L0308
// rustc E0596: an FnMut behind an Rc cannot be borrowed as mutable.
use std::rc::Rc;
fn call<F: FnMut() -> u8>(f: F) -> u8 { let p = Rc::new(f); p() }
fn main() { let mut n = 0; println!("{}", call(move || { n += 1; n })); }
