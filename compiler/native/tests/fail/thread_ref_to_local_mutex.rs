// expect: L0306
use std::thread;
use std::sync::Mutex;
fn main() { let m = Mutex::new(1); let r = &m; thread::spawn(move || { *r.lock().unwrap() += 1; }).join().unwrap(); }
