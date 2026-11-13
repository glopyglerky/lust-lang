// expect: L0309
use std::rc::Rc;
fn main() { let r = Rc::new((1, 2)); let mut f = || { r.0 += 1; }; f(); }
