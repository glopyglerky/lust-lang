// expect: L0311
use std::rc::Rc;
fn main() { let r = Rc::new((String::new(), 1)); let f = || r.0.len(); drop(r); println!("{}", f()); }
