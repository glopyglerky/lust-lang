// expect: L0612
use std::sync::Mutex;
fn need<T: Send>(_: T) {}
fn main() { let m = Mutex::new(1); need(m.lock().unwrap()); }
