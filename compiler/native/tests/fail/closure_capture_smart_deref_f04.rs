// expect: L0308
use std::sync::Mutex;
fn main() { let m = Mutex::new((1, 2)); let g = m.lock().unwrap(); let mut f = || { g.0 += 1; }; f(); }
