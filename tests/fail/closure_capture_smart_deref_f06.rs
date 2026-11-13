// expect: L0303
use std::sync::Mutex;
fn main() { let m = Mutex::new((1, 2)); let mut g = m.lock().unwrap(); let mut f = || { g.0 += 1; }; let x = g.1; f(); println!("{}", x); }
