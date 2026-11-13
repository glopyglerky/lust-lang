// expect: L0306
use std::sync::{Arc, Mutex};
use std::thread;
fn main() { let st = Arc::new((Mutex::new(0), 1)); let h = thread::spawn(|| { *st.0.lock().unwrap() += 1; }); h.join().unwrap(); }
