// expect: L0612
use std::sync::RwLock;
use std::cell::RefCell;
fn need<T: Sync>(_: &T) {}
fn main() { let m = RwLock::new(RefCell::new(1)); need(&m); }
