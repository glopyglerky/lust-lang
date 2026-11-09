// expect: L0612
use std::sync::Arc;
use std::cell::Cell;
fn need<T: Send>(_: &T) {}
fn main() { let a = Arc::new(Cell::new(1)); need(&a); println!("{}", a.get()); }
