// expect: L0311
use std::sync::Mutex;
fn main() { let m = Mutex::new(1); let g = m.lock().unwrap(); drop(m); println!("{}", *g); }
