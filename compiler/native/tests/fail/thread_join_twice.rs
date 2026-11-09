// expect: L0301
use std::thread;
fn main() { let h = thread::spawn(|| 1); let a = h.join(); let b = h.join(); println!("{:?} {:?}", a, b); }
