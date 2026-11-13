// expect: L0306
use std::thread;
fn main() { let v = vec![1]; let h = thread::spawn(|| v.len()); println!("{}", h.join().unwrap()); }
