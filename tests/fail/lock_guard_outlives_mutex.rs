// expect: L0306
use std::sync::Mutex;
fn main() { let g; { let m = Mutex::new(1); g = m.lock().unwrap(); } println!("{}", g); }
