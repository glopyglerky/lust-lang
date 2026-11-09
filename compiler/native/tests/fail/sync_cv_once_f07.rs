// expect: L0110
use std::sync::{Mutex, Condvar};
fn main() { let m = Mutex::new(vec![1]); let c = Condvar::new(); let g = c.wait_while(m.lock().unwrap(), |v| { v.push(2); false }).unwrap(); let s: &str = g; }
