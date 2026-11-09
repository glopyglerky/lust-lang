// expect: L0301
use std::sync::{Mutex, Condvar};
fn main() { let m = Mutex::new(0); let c = Condvar::new(); let g = m.lock().unwrap(); let g2 = c.wait(g).unwrap(); println!("{} {}", g2, g); }
