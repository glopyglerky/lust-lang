// expect: L0612
use std::thread;
use std::sync::Arc;
use std::cell::RefCell;
fn main() { let c = Arc::new(RefCell::new(1)); let c2 = Arc::clone(&c); thread::spawn(move || { *c2.borrow_mut() += 1; }).join().unwrap(); }
