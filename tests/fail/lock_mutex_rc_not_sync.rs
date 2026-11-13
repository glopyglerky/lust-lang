// expect: L0612
use std::sync::Mutex;
use std::rc::Rc;
fn need<T: Sync>(_: &T) {}
fn main() { let m = Mutex::new(Rc::new(1)); need(&m); }
