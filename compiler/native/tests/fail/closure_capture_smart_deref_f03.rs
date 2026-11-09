// expect: L0301
use std::rc::Rc;
fn main() { let r = Rc::new((String::new(), 1)); let f = move || r.0.len(); println!("{} {}", f(), r.1); }
