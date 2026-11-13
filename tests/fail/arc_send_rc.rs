// expect: L0612
use std::sync::Arc;
use std::rc::Rc;
fn need<T: Send>(_: &T) {}
fn main() { let a = Arc::new(Rc::new(1)); need(&a); println!("{}", a); }
