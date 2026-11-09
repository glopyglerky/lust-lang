use std::sync::Mutex;
use std::cell::Cell;
fn need<T: Sync>(_: &T) {}
fn main() { let m = Mutex::new(Cell::new(1)); need(&m); println!("{}", m.lock().unwrap().get()); }
